use std::sync::Arc;
use std::time::Duration;

use actix_web::{get, web, HttpResponse};
use anyhow;
use chrono::Utc;
use futures::lock::Mutex;
use juniper::{FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject};
use oauth2::basic::{BasicClient, BasicTokenResponse, BasicTokenType};
use oauth2::reqwest::async_http_client;
use oauth2::{
  AccessToken, AsyncCodeTokenRequest, AsyncRefreshTokenRequest, AuthUrl, AuthorizationCode,
  ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier,
  RedirectUrl, RefreshToken, RequestTokenError, Scope, StandardTokenResponse, TokenResponse,
  TokenUrl,
};
use reqwest;
use serde::Deserialize;
use url::Url;

use crate::config::CONFIG;

const SAMPLE_WORKSHEET: &str =
  "https://docs.google.com/spreadsheets/d/1iqJIuqxulhM0VRVfXaQOxj9rSP5zW59UE3PeOk8Vhn0/edit#gid=0";
const SHEETS_READ_URL: &str =
  "https://sheets.googleapis.com/v4/spreadsheets/spreadsheetId/values:batchGet";

fn get_spreadsheet_id(sheet_url: &str) -> &str {
  let mut split_iter = sheet_url.split("/").skip(5);
  split_iter.next().unwrap()
}

#[derive(Deserialize)]
struct OAuthRequest {
  code: Option<String>,
  state: Option<String>,
}

#[derive(Deserialize)]
pub struct GoogleSheetsParams {
  sheet_name: String,
  // Allow for pulling of entire sheet
  range: Option<String>,
}

#[derive(Clone, Debug)]
pub struct OAuthService {
  pub client: BasicClient,
  access_token: Option<AccessToken>,
  refresh_token: Option<RefreshToken>,
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
    let oauth_config = &CONFIG.get().unwrap().oauth;
    let client_id = ClientId::new(oauth_config.client_id.clone());
    let client_secret = ClientSecret::new(oauth_config.client_secret.clone());
    let auth_url =
      AuthUrl::new(oauth_config.auth_url.clone()).expect("Error parsing OAuth authorization url.");
    let token_url =
      TokenUrl::new(oauth_config.token_url.clone()).expect("Error parsing OAuth token url.");
    let auth_redirect_uri = RedirectUrl::new(oauth_config.auth_redirect_url.clone())
      .expect("Error parsing OAuth redirect url.");
    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
      .set_redirect_url(
        RedirectUrl::new(auth_redirect_uri.to_string()).expect("Invalid redirect URL."),
      );

    OAuthService {
      client,
      access_token: None,
      refresh_token: None,
      access_token_expiration: 0,
      refresh_token_expiration: 0,
    }
  }

  async fn fetch_token(&mut self, code: AuthorizationCode) -> Result<AccessToken, anyhow::Error> {
    let token = self
      .client
      .exchange_code(code)
      .request_async(async_http_client)
      .await
      .unwrap();

    // TODO: more sensible default == 0? err on the side of caution
    let access_expires_in = token
      .expires_in()
      .unwrap_or(Duration::new(3600, 0))
      .as_secs() as i64;
    self.access_token_expiration = Utc::now().timestamp() + access_expires_in;
    self.access_token = Some(token.access_token().clone());
    self.refresh_token = token.refresh_token().cloned();

    Ok(self.access_token.clone().unwrap())
  }

  // Retrieves access token from client. Refreshes it with Google's server if necessary.
  // Note that this does not automatically fetch the access token if not present -- the authorization code is needed for that.
  async fn get_token(&mut self) -> Option<AccessToken> {
    if let Some(_) = self.access_token.as_ref() {
      // Refresh token protocol. See: https://developers.google.com/identity/protocols/oauth2/web-server#offline
      let now = Utc::now().timestamp();
      if now >= self.access_token_expiration && self.refresh_token.is_some() {
        println!("Access token expired -- refreshing");
        let refresh_token = self.refresh_token.as_ref()?;
        let new_token = self
          .client
          .exchange_refresh_token(refresh_token)
          .request_async(async_http_client)
          .await
          .ok()?;

        let expires_in = new_token
          .expires_in()
          .unwrap_or(Duration::new(3600, 0))
          .as_secs() as i64;
        self.access_token = Some(new_token.access_token().clone());
        self.access_token_expiration = now + expires_in;
      }

      self.access_token.clone()
    } else {
      None
    }
  }
}

#[get("/get_sheets")]
pub async fn get_sheets_value(
  data: web::Data<Arc<Mutex<OAuthService>>>,
  params: web::Query<GoogleSheetsParams>,
) -> HttpResponse {
  let sheet_url = SHEETS_READ_URL.replace("spreadsheetId", get_spreadsheet_id(SAMPLE_WORKSHEET));
  let client = reqwest::Client::new();
  let token = data
    .lock()
    .await
    .get_token()
    .await
    .expect("No access token found, please authenticate before querying.");
  let sheet_name = params.sheet_name.clone();
  let range = if let Some(range) = params.range.as_ref() {
    format!("{}!{}", sheet_name, range)
  } else {
    sheet_name
  };

  let resp = client
    .get(sheet_url)
    .bearer_auth(token.secret())
    .query(&[("ranges", range), ("majorDimension", "ROWS".to_string())])
    .send()
    .await
    .unwrap();

  let text_resp = resp.text().await.unwrap();
  // let sheets_obj: SheetsObject = serde_json::from_str(&text_resp).unwrap();
  // let google_sheet: GoogleRowSheet = sheets_obj.into();

  HttpResponse::Ok().body(text_resp)
}

// Endpoint to redirect after initial authorization with Google server
#[get("/auth")]
async fn exchange_token(
  data: web::Data<Arc<Mutex<OAuthService>>>,
  params: web::Query<OAuthRequest>,
) -> HttpResponse {
  let mut oauth = data.lock().await;
  let token = if let Some(auth_code) = params.code.as_ref() {
    let code = AuthorizationCode::new(auth_code.clone());
    oauth.fetch_token(code).await.unwrap()
  } else {
    oauth.get_token().await.unwrap()
  };
  // let state = CsrfToken::new(params.state.cloned());

  HttpResponse::Ok().body(serde_json::to_string(&token).unwrap())
}

// Handler for adding new data sources
#[get("/add")]
pub async fn add_data_source(data: web::Data<Arc<Mutex<OAuthService>>>) -> HttpResponse {
  let mut oauth = data.lock().await;
  let oauth_config = &CONFIG.get().unwrap().oauth;

  if let Some(_) = oauth.get_token().await {
    HttpResponse::Found()
      // TODO: handle this case more elegantly -- any way to retrieve redirect_uri directly from client?
      .append_header((
        reqwest::header::LOCATION,
        oauth_config.auth_redirect_url.clone(),
      ))
      .finish()
  } else {
    // let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (authorize_url, _csrf_token) = oauth
      .client
      .authorize_url(CsrfToken::new_random)
      //.set_pkce_challenge(pkce_challenge)
      .add_scope(Scope::new(oauth_config.scope.clone()))
      // For Google to return a refresh token
      .add_extra_param("access_type", "offline")
      .url();

    HttpResponse::Found()
      .append_header((reqwest::header::LOCATION, authorize_url.to_string()))
      .finish()
  }
}
