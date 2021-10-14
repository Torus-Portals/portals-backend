use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use actix_web::{get, web, HttpResponse};
use anyhow::{anyhow, Result};
use chrono::Utc;
use futures::lock::Mutex;
use juniper::GraphQLObject;
use reqwest;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::config::{self, CONFIG};

const SHEETS_READ_URL: &str =
  "https://sheets.googleapis.com/v4/spreadsheets/spreadsheetId/values:batchGet";
const SPREADSHEET_SHEETS_URL: &str = "https://sheets.googleapis.com/v4/spreadsheets/spreadsheetId";
const DRIVE_FILES_URL: &str = "https://www.googleapis.com/drive/v3/files";
const OPENID_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";

fn get_spreadsheet_id(sheet_url: &str) -> &str {
  let mut split_iter = sheet_url.split("/").skip(5);
  split_iter.next().unwrap()
}

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

#[derive(Deserialize)]
pub struct GoogleSheetsParams {
  sheet_url: String,
  sheet_name: Option<String>,
  // Allow for pulling of entire sheet
  range: Option<String>,
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

#[derive(GraphQLObject, Debug, Deserialize)]
pub struct GoogleSheetsSpreadsheet {
  id: String,
  name: String,
}

#[derive(GraphQLObject, Debug, Deserialize)]
pub struct GoogleSheetsSheetDimensions {
  row_dimensions: Vec<String>,
  col_dimensions: Vec<String>,
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

  // pub fn exchange_code(&mut self, code: &str) -> Result<bool> {
  //   //
  //   self.codes.push(code.to_owned());

  //   Ok(true)
  // }

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

    // let resp_string = resp.text().await?;
    // dbg!(&resp_string);
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
    let client = reqwest::Client::new();

    let resp = client
      .get(OPENID_URL)
      .bearer_auth(gsheets_token.access_token.clone())
      .send()
      .await?;

    let resp_string = resp.text().await?;
    let resp_value: Value = serde_json::from_str(&resp_string)?;

    Ok(resp_value["email"].as_str().unwrap_or("").to_string())
  }

  pub async fn list_spreadsheets(
    &self,
    integration_id: Uuid,
  ) -> Result<Vec<GoogleSheetsSpreadsheet>> {
    let client = reqwest::Client::new();

    let token = self.get_token(integration_id).await?;
    let resp = client
      .get(DRIVE_FILES_URL)
      .bearer_auth(token.access_token.clone())
      .query(&[("q", "mimeType='application/vnd.google-apps.spreadsheet'")])
      .send()
      .await?;

    let resp_string = resp.text().await?;
    let resp_value: Value = serde_json::from_str(&resp_string)?;
    let resp_files = resp_value["files"]
      .as_array()
      .cloned()
      .unwrap_or_else(|| Vec::new());
    let spreadsheet_names: Vec<String> = resp_files
      .iter()
      .map(|value| value["name"].as_str().unwrap_or("").to_string())
      .collect();
    let spreadsheet_ids: Vec<String> = resp_files
      .iter()
      .map(|value| value["id"].as_str().unwrap_or("").to_string())
      .collect();

    Ok(
      spreadsheet_names
        .into_iter()
        .zip(spreadsheet_ids.into_iter())
        .map(|(name, id)| GoogleSheetsSpreadsheet { name, id })
        .collect(),
    )
  }

  pub async fn list_spreadsheet_sheets_names(
    &self,
    integration_id: Uuid,
    spreadsheet_id: String,
  ) -> Result<Vec<String>> {
    let client = reqwest::Client::new();

    let token = self.get_token(integration_id).await?;
    let resp = client
      .get(SPREADSHEET_SHEETS_URL.replace("spreadsheetId", spreadsheet_id.as_str()))
      .bearer_auth(token.access_token.clone())
      .send()
      .await?;

    let resp_string = resp.text().await?;
    let spreadsheet: Value = serde_json::from_str(&resp_string)?;
    let sheet_names: Vec<String> = spreadsheet["sheets"]
      .as_array()
      .unwrap_or(&Vec::new())
      .into_iter()
      .map(|sheet| {
        sheet["properties"]["title"]
          .as_str()
          .unwrap_or("")
          .to_string()
      })
      .collect();

    Ok(sheet_names)
  }

  pub async fn fetch_sheet_dimensions(
    &self,
    integration_id: Uuid,
    spreadsheet_id: String,
    sheet_name: String,
  ) -> Result<GoogleSheetsSheetDimensions> {
    let client = reqwest::Client::new();

    let token = self.get_token(integration_id).await?;
    let resp = client
      .get(SHEETS_READ_URL.replace("spreadsheetId", spreadsheet_id.as_str()))
      .bearer_auth(token.access_token.clone())
      .query(&[("ranges", sheet_name)])
      .send()
      .await?;

    let resp_string = resp.text().await?;
    let sheet: SheetsObject = serde_json::from_str(&resp_string)?;

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
    let client = reqwest::Client::new();

    let token = self.get_token(integration_id).await?;
    let resp = client
      .get(SHEETS_READ_URL.replace("spreadsheetId", spreadsheet_id.as_str()))
      .bearer_auth(token.access_token.clone())
      .query(&[("ranges", sheet_range)])
      .send()
      .await?;

    let resp_string = resp.text().await?;
    let sheet: SheetsObject = serde_json::from_str(&resp_string)?;

    Ok(sheet.value_ranges[0].values[0][0].clone())
  }
}

// // Wrapper around GET request to endpoint -- for external services
// pub async fn fetch_sheets_value(
//   sheet_url: String,
//   sheet_name: String,
//   range: Option<String>,
// ) -> SheetsObject {
//   let client = reqwest::Client::new();
//   let mut req = client
//     .get("http://localhost:8088/get_sheets_value")
//     .query(&[("sheet_url", sheet_url), ("sheet_name", sheet_name)]);

//   // If no range argument provided, fetches all cells in entire sheet.
//   if let Some(range_str) = range.as_ref() {
//     req = req.query(&[("range", range_str.as_str())]);
//   }

//   let sheet = req.send().await.unwrap();

//   serde_json::from_str(&sheet.text().await.unwrap()).unwrap()
// }

// #[get("/get_sheets_value")]
// pub async fn get_sheets_value(
//   data: web::Data<Arc<Mutex<OAuthService>>>,
//   params: web::Query<GoogleSheetsParams>,
// ) -> HttpResponse {
//   let sheet_url = SHEETS_READ_URL.replace(
//     "spreadsheetId",
//     get_spreadsheet_id(params.sheet_url.as_str()),
//   );
//   let client = reqwest::Client::new();
//   let token = data
//     .lock()
//     .await
//     .get_token()
//     .await
//     .expect("No access token found, please authenticate before querying.");
//   let sheet_name = params.sheet_name.clone().unwrap_or("Sheet1".to_string());
//   let range = if let Some(range) = params.range.as_ref() {
//     format!("{}!{}", sheet_name, range)
//   } else {
//     sheet_name
//   };

//   let resp = client
//     .get(sheet_url)
//     .bearer_auth(token)
//     .query(&[("ranges", range), ("majorDimension", "ROWS".to_string())])
//     .send()
//     .await
//     .unwrap();

//   let text_resp = resp.text().await.unwrap();
//   // let sheets_obj: SheetsObject = serde_json::from_str(&text_resp).unwrap();
//   // let google_sheet: GoogleRowSheet = sheets_obj.into();

//   HttpResponse::Ok().body(text_resp)
// }

// #[get("/get_sheets")]
// pub async fn get_sheets(
//   data: web::Data<Arc<Mutex<OAuthService>>>,
//   params: web::Query<GoogleSheetsParams>,
// ) -> HttpResponse {
//   let sheet_url = SPREADSHEET_SHEETS_URL.replace(
//     "spreadsheetId",
//     get_spreadsheet_id(params.sheet_url.as_str()),
//   );
//   let client = reqwest::Client::new();
//   let token = data
//     .lock()
//     .await
//     .get_token()
//     .await
//     .expect("No access token found, please authenticate before querying.");

//   let resp = client
//     .get(sheet_url)
//     .bearer_auth(token)
//     .send()
//     .await
//     .unwrap();

//   let spreadsheet_value: Value =
//     serde_json::from_str(&resp.text().await.unwrap()).expect("Unable to parse JSON response.");
//   let sheet_names: Vec<Value> = spreadsheet_value["sheets"]
//     .as_array()
//     .unwrap_or(&Vec::new())
//     .into_iter()
//     .map(|sheet| sheet["properties"]["title"].clone())
//     .collect();

//   HttpResponse::Ok().body(serde_json::to_vec(&sheet_names).unwrap())
// }

// Endpoint for internal exchange of token with Google's server after obtaining authorization code
// #[get("/auth")]
// async fn exchange_token(
//   data: web::Data<Arc<Mutex<OAuthService>>>,
//   params: web::Query<OAuthRequest>,
// ) -> HttpResponse {
//   let mut oauth = data.lock().await;
//   let _token = if let Some(auth_code) = &params.code {
//     oauth.fetch_token(auth_code.to_owned()).await.unwrap()
//   } else {
//     oauth.get_token().await.unwrap()
//   };
//   // let state = CsrfToken::new(params.state.cloned());

//   // HttpResponse::Ok().body(serde_json::to_string(&token).unwrap())
//   // TODO: Redirect to appropriate page resource (depending on front-end)
//   HttpResponse::Found()
//     .append_header((reqwest::header::LOCATION, "https://www.portals-dev.rocks"))
//     .finish()
// }

// // Handler for adding new data sources
// #[get("/add")]
// pub async fn add_data_source(data: web::Data<Arc<Mutex<OAuthService>>>) -> HttpResponse {
//   let mut oauth = data.lock().await;
//   let oauth_config = &CONFIG.get().unwrap().oauth;

//   match oauth.get_token().await {
//     Ok(_token) => HttpResponse::Found()
//       .append_header((
//         reqwest::header::LOCATION,
//         oauth_config.auth_redirect_url.clone(),
//       ))
//       .finish(),
//     Err(_) => {
//       let authorize_url = format!(
//         "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline",
//         oauth_config.auth_url,
//         oauth_config.client_id,
//         oauth_config.auth_redirect_url,
//         oauth_config.scope,
//       );

//       HttpResponse::Found()
//         .append_header((reqwest::header::LOCATION, authorize_url))
//         .finish()
//     }
//   }
// }
