use reqwest;
use std::env;

use percent_encoding::{ utf8_percent_encode, NON_ALPHANUMERIC };

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth0TokenResponse {
  pub access_token: String,
  pub scope: String,
  pub expires_in: isize,
  pub token_type: String,
}

// TODO: Should cache this token and only refetch it when it expires.
// TODO: Move at least client_id/client_secret somewhere else as static.
pub fn get_auth0_token() -> Result<Auth0TokenResponse, reqwest::Error> {
  let client_id = env::var("AUTH0_CLIENT_ID")
    .expect("Unable to get AUTH0_CLIENT_ID env var.");

  let client_secret = env::var("AUTH0_CLIENT_SECRET")
    .expect("Unable to get AUTH0_CLIENT_SECRET env var.");
  
  let audience = env::var("AUTH0_AUDIENCE")
    .expect("AUTH0_AUDIENCE env var not found.");

  let token_endpoint = env::var("AUTH0_TOKEN_ENDPOINT")
    .expect("AUTH0_TOKEN_ENDPOINT env var not found");

  let params = [
    ("grant_type", "client_credentials"),
    ("client_id", &client_id),
    ("client_secret", &client_secret),
    ("audience", &audience)
  ];

  let auth0_token_response: Auth0TokenResponse = reqwest::Client::new()
    .post(&token_endpoint)
    .form(&params)
    .send()?
    .json()?;

  Ok(auth0_token_response)
}

pub fn get_auth0_user(auth0id: &str) -> Result<Auth0User, reqwest::Error> {
  let token = get_auth0_token()?;

  let auth_string = format!("{} {}", token.token_type, token.access_token);

  let auth0_path = env::var("AUTH0_USER_ENDPOINT")
    .expect("AUTH0_USER_ENDPOINT env var not found.");
  // let auth0_path = String::from("https://torus-rocks.auth0.com/api/v2/users/");

  let url = format!("{}{}", auth0_path, utf8_percent_encode(&auth0id, NON_ALPHANUMERIC).to_string());

  let auth0_user: Auth0User = reqwest::Client::new()
    .get(&url)
    .header("Authorization", auth_string)
    .send()?
    .json()?;

  Ok(auth0_user)
}