use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use actix_web::{get, web, HttpResponse};
use anyhow::{anyhow, Result};
use chrono::Utc;
use futures::lock::Mutex;
use juniper::GraphQLObject;
use reqwest;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use uuid::Uuid;

use crate::config::{self, CONFIG};
use crate::graphql::schema::integrations::google_sheets::{
  GoogleSheetsSheetDimensions, GoogleSheetsSpreadsheet,
};

use super::meltano_service::{ExtractorConfigData, GoogleSheetsConfigData};

#[derive(Deserialize)]
struct OAuthRequest {
  code: Option<String>,
  state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
  access_token: String,
  expires_in: i64,
  refresh_token: String,
  scope: String,
  token_type: String,
}

#[derive(Deserialize)]
struct OAuthRefreshTokenResponse {
  access_token: String,
  expires_in: i64,
  scope: String,
  token_type: String,
}

#[derive(Clone, Debug)]
pub struct OAuthService {
  access_token: Option<String>,
  refresh_token: Option<String>,
  access_token_expiration: i64,
  refresh_token_expiration: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetsCells {
  pub range: String,
  pub major_dimension: String,
  pub values: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetsObject {
  pub spreadsheet_id: String,
  pub value_ranges: Vec<SheetsCells>,
}

impl OAuthService {
  pub fn new() -> Self {
    OAuthService {
      access_token: None,
      refresh_token: None,
      access_token_expiration: 0,
      refresh_token_expiration: 0,
    }
  }

  async fn fetch_token(&mut self, code: String) -> Result<String, anyhow::Error> {
    let oauth_config = &CONFIG.get().unwrap().oauth;
    let client = reqwest::Client::new();
    let form_params = [
      ("code", code.as_str()),
      ("client_id", oauth_config.client_id.as_str()),
      ("client_secret", oauth_config.client_secret.as_str()),
      ("grant_type", "authorization_code"),
      ("redirect_uri", "http://localhost:8088/auth"),
    ];

    let resp = client
      .post(oauth_config.token_url.clone())
      .header("Content-Type", "application/x-www-form-urlencoded")
      .form(&form_params)
      .send()
      .await?;

    let token_resp = resp.json::<OAuthTokenResponse>().await?;

    self.access_token_expiration = Utc::now().timestamp() + token_resp.expires_in;
    self.access_token = Some(token_resp.access_token.clone());
    self.refresh_token = Some(token_resp.refresh_token);

    Ok(token_resp.access_token)
  }

  // Retrieves access token from client. Refreshes it with Google's server if necessary.
  // Note that this does not automatically fetch the access token if not present -- the authorization code is needed for that.
  async fn get_token(&mut self) -> Result<String, anyhow::Error> {
    // Refresh token protocol. See: https://developers.google.com/identity/protocols/oauth2/web-server#offline
    let now = Utc::now().timestamp();
    if self.access_token_expiration != 0 && now >= self.access_token_expiration {
      println!("Access token expired -- attempting to refresh");
      let oauth_config = &CONFIG.get().unwrap().oauth;
      let refresh_token = self.refresh_token.as_deref().ok_or(anyhow::anyhow!(
        "No refresh token found -- unable to exchange for new access token."
      ))?;
      let client = reqwest::Client::new();

      let form_params = [
        ("client_id", oauth_config.client_id.as_str()),
        ("client_secret", oauth_config.client_secret.as_str()),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
      ];

      let resp = client
        .post(oauth_config.token_url.clone())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&form_params)
        .send()
        .await?;

      let refresh_token_resp = resp.json::<OAuthRefreshTokenResponse>().await?;
      self.access_token = Some(refresh_token_resp.access_token.clone());
      self.access_token_expiration = now + refresh_token_resp.expires_in;

      Ok(refresh_token_resp.access_token)
    } else {
      self.access_token.clone().ok_or(anyhow::anyhow!(
        "No OAuth access token found. Proceeding to authorization."
      ))
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct GoogleSheetsToken {
  access_token: String,
  refresh_token: String,
  expires_in: i64,
}

#[derive(Debug, Deserialize)]
pub struct GoogleUser {
  email: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleDriveSpreadsheets {
  files: Vec<GoogleSheetsSpreadsheet>,
}

#[derive(Debug)]
pub struct GoogleSpreadsheetSheets {
  sheets: Vec<String>,
}

impl<'de> Deserialize<'de> for GoogleSpreadsheetSheets {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    struct OuterSheets {
      sheets: Vec<SheetProperties>,
    }

    #[derive(Deserialize)]
    struct SheetProperties {
      properties: InnerProperties,
    }

    #[derive(Deserialize)]
    struct InnerProperties {
      title: String,
    }

    let helper = OuterSheets::deserialize(deserializer)?;
    Ok(GoogleSpreadsheetSheets {
      sheets: helper
        .sheets
        .into_iter()
        .map(|p| p.properties.title)
        .collect(),
    })
  }
}

pub struct GoogleSheetsService {
  pub tokens: HashMap<Uuid, GoogleSheetsToken>,
}

impl GoogleSheetsService {
  pub fn new() -> Self {
    GoogleSheetsService {
      tokens: HashMap::new(),
    }
  }

  pub async fn exchange_code(&self, code: String) -> Result<GoogleSheetsToken> {
    let oauth_config = &CONFIG.get().unwrap().oauth;
    let client = reqwest::Client::new();
    let form_params = [
      ("code", code.as_str()),
      ("client_id", oauth_config.client_id.as_str()),
      ("client_secret", oauth_config.client_secret.as_str()),
      ("grant_type", "authorization_code"),
      ("redirect_uri", oauth_config.auth_redirect_url.as_str()),
    ];

    let resp = client
      .post(oauth_config.token_url.clone())
      .header("Content-Type", "application/x-www-form-urlencoded")
      .form(&form_params)
      .send()
      .await?;

    // TODO: store the token_resp locally so that it may be used again.
    //       need to figure out what the best way to look up the token will be.
    //       Maybe the portal id? Maybe this will be tied to an instance of an integration?
    let token_resp = resp.json::<OAuthTokenResponse>().await?;
    let gsheets_token = GoogleSheetsToken {
      access_token: token_resp.access_token,
      refresh_token: token_resp.refresh_token,
      expires_in: token_resp.expires_in,
    };

    dbg!(&gsheets_token);

    Ok(gsheets_token)
  }

  pub async fn store_token(
    &mut self,
    integration_id: Uuid,
    gsheets_token: GoogleSheetsToken,
  ) -> Result<bool> {
    Ok(self.tokens.insert(integration_id, gsheets_token).is_none())
  }

  pub async fn get_token(&self, integration_id: Uuid) -> Result<&GoogleSheetsToken> {
    self.tokens.get(&integration_id).ok_or(anyhow!(format!(
      "No access token associated with this integration: {}",
      integration_id
    )))
  }

  pub async fn get_user_email(&self, gsheets_token: &GoogleSheetsToken) -> Result<String> {
    let config = config::server_config();
    let client = reqwest::Client::new();

    let resp = client
      .get(config.oauth.endpoints.openid_url.as_str())
      .bearer_auth(gsheets_token.access_token.clone())
      .send()
      .await?;

    let google_user: GoogleUser = resp.json().await?;

    Ok(google_user.email.clone())
  }

  pub async fn list_spreadsheets(
    &self,
    integration_id: Uuid,
  ) -> Result<Vec<GoogleSheetsSpreadsheet>> {
    let config = config::server_config();
    let client = reqwest::Client::new();

    let token = self.get_token(integration_id).await?;
    let resp = client
      .get(config.oauth.endpoints.drive_files_url.as_str())
      .bearer_auth(token.access_token.clone())
      .query(&[("q", "mimeType='application/vnd.google-apps.spreadsheet'")])
      .send()
      .await?;

    let gdrive_files: GoogleDriveSpreadsheets = resp.json().await?;

    Ok(gdrive_files.files)
  }

  pub async fn list_spreadsheet_sheets_names(
    &self,
    integration_id: Uuid,
    spreadsheet_id: String,
  ) -> Result<Vec<String>> {
    let config = config::server_config();
    let client = reqwest::Client::new();

    let token = self.get_token(integration_id).await?;
    let resp = client
      .get(
        config
          .oauth
          .endpoints
          .spreadsheets_sheets_url
          .replace("spreadsheetId", spreadsheet_id.as_str()),
      )
      .bearer_auth(token.access_token.clone())
      .send()
      .await?;

    let sheets: GoogleSpreadsheetSheets = resp.json().await?;

    Ok(sheets.sheets)
  }

  pub async fn get_sheets_meltano_config(
    &self,
    integration_id: Uuid,
    spreadsheet_id: String,
    sheet_name: String,
  ) -> Result<ExtractorConfigData> {
    let token = self.get_token(integration_id).await?;
    let refresh_token = token.refresh_token.clone();

    Ok(ExtractorConfigData::GoogleSheets(GoogleSheetsConfigData {
      refresh_token,
      spreadsheet_id,
      sheet_name,
    }))
  }

  pub async fn fetch_sheet_dimensions(
    &self,
    integration_id: Uuid,
    spreadsheet_id: String,
    sheet_name: String,
  ) -> Result<GoogleSheetsSheetDimensions> {
    let config = config::server_config();
    let client = reqwest::Client::new();

    let token = self.get_token(integration_id).await?;
    let resp = client
      .get(
        config
          .oauth
          .endpoints
          .sheets_read_url
          .replace("spreadsheetId", spreadsheet_id.as_str()),
      )
      .bearer_auth(token.access_token.clone())
      .query(&[("ranges", sheet_name)])
      .send()
      .await?;

    let sheet: SheetsObject = resp.json().await?;
    dbg!(&sheet);

    let row_dimensions: Vec<String> = sheet.value_ranges[0]
      .values
      .iter()
      .skip(1)
      .map(|row| row[0].clone())
      .collect();
    let col_dimensions: Vec<String> = sheet.value_ranges[0].values[0].clone();

    Ok(GoogleSheetsSheetDimensions {
      row_dimensions,
      col_dimensions,
    })
  }

  pub async fn fetch_sheet_value(
    &self,
    integration_id: Uuid,
    spreadsheet_id: String,
    sheet_range: String,
  ) -> Result<String> {
    let config = config::server_config();
    let client = reqwest::Client::new();

    let token = self.get_token(integration_id).await?;
    let resp = client
      .get(
        config
          .oauth
          .endpoints
          .sheets_read_url
          .replace("spreadsheetId", spreadsheet_id.as_str()),
      )
      .bearer_auth(token.access_token.clone())
      .query(&[("ranges", sheet_range)])
      .send()
      .await?;

    let sheet: SheetsObject = resp.json().await?;
    dbg!(&sheet);

    Ok(sheet.value_ranges[0].values[0][0].clone())
  }
}
