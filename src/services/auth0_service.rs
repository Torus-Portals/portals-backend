use crate::config;
use anyhow;
use chrono::Utc;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auth0User {
  pub email_verified: bool,
  pub email: String,
  pub updated_at: String,
  pub user_id: String,
  pub name: String,
  pub picture: String,
  pub nickname: String,
  pub created_at: String,
  pub last_ip: String,
  pub last_login: String,
  pub logins_count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auth0Token {
  pub access_token: String,
  pub scope: String,
  pub token_type: String,
  pub expires_in: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FetchTokenPayload {
  pub client_id: String,
  pub client_secret: String,
  pub grant_type: String,
  pub audience: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Auth0Service {
  pub auth0_client_id: String,
  pub auth0_client_secret: String,
  pub auth_api_url: String,
  pub token_url: String,
  auth_token: Option<Auth0Token>,
  token_expiration: i64,
}

impl Auth0Service {
  pub fn new() -> Self {
    let auth0 = &config::server_config().auth0;
    Auth0Service {
      auth0_client_id: auth0
        .client_id
        .to_owned(),
      auth0_client_secret: auth0
        .client_secret
        .to_owned(),
      auth_api_url: auth0
        .api_endpoint
        .to_owned(),
      token_url: auth0
        .token_endpoint
        .to_owned(),
      auth_token: None,
      token_expiration: 0,
    }
  }

  pub async fn fetch_token(&mut self) -> Result<Auth0Token, anyhow::Error> {
    let payload = FetchTokenPayload {
      client_id: self
        .auth0_client_id
        .to_owned(),
      client_secret: self
        .auth0_client_secret
        .to_owned(),
      grant_type: String::from("client_credentials"),
      audience: String::from(&self.auth_api_url),
    };

    let client = reqwest::Client::new();

    let resp = client
      .post(&self.token_url)
      .header("Content-Type", "application/json")
      .json(&payload)
      .send()
      .await?;

    let token_resp = resp
      .json::<Auth0Token>()
      .await?;

    self.auth_token = Some(token_resp.clone());

    let now = Utc::now().timestamp();
    self.token_expiration = now + (token_resp.expires_in * 1000);

    Ok(token_resp)
  }

  pub async fn get_token(&mut self) -> Result<String, anyhow::Error> {
    match self
      .auth_token
      .as_ref()
    {
      Some(at) => {
        let current_time = Utc::now().timestamp() + at.expires_in;

        if current_time > self.token_expiration {
          self
            .fetch_token()
            .await?;
        }
      }
      None => {
        self
          .fetch_token()
          .await?;
      }
    };

    let access_token = self
      .auth_token
      .to_owned()
      .ok_or(anyhow::Error::msg("unable to get auth0 access_token"))?
      .access_token;

    Ok(access_token)
  }

  pub async fn get_auth0_user(&mut self, auth0_id: &str) -> Result<Auth0User, anyhow::Error> {
    let client = reqwest::Client::new();

    let url = format!(
      "{}users/{}",
      self.auth_api_url,
      utf8_percent_encode(&auth0_id, NON_ALPHANUMERIC).to_string()
    );

    let access_token = self
      .get_token()
      .await?;

    let resp = client
      .get(&url)
      .header("Authorization", format!("Bearer {}", access_token))
      .send()
      .await?;

    let auth0user = resp
      .json::<Auth0User>()
      .await?;

    Ok(auth0user)
  }
}
