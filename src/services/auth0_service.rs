use reqwest::{ Client as req, StatusCode };
// use reqwest::StatusCode;
use std::env;
use std::error::Error;
use std::fmt;
use serde::de::DeserializeOwned;

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
  // pub scope: String,
  pub expires_in: isize,
  pub token_type: String,
}

#[derive(Debug)]
pub struct Auth0RequestError {
  payload: String,
}

impl fmt::Display for Auth0RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let p = self.payload
        write!(f, "Auth0RequestError!")
    }
}

impl Error for Auth0RequestError {}

impl From<reqwest::Error> for Auth0RequestError {
  fn from(err: reqwest::Error) -> Self {
    println!("reqwest error: {:?}", err);
    Auth0RequestError {
      payload: err.to_string()
    }
  }
}

pub fn handle_auth0_response<T: DeserializeOwned>(mut resp: reqwest::Response) -> Result<T, Auth0RequestError> {
  match resp.status() {
    StatusCode::OK => {
      let json: T = resp.json()?;
      Ok(json)
    },
    StatusCode::FORBIDDEN => {
      println!("forbidden!!!");
      let p = resp.text()?;

      Err(Auth0RequestError {
        payload: p
      })
    },
    _ => {
      println!("baad status code...");
      Err(Auth0RequestError {
        payload: String::from("other")
      })
    }
  }
}

// TODO: Should cache this token and only refetch it when it expires.
// TODO: Move at least client_id/client_secret somewhere else as static.
pub fn get_auth0_token() -> Result<Auth0TokenResponse, Auth0RequestError> {
// pub fn get_auth0_token() -> Result<Auth0TokenResponse, reqwest::Error> {
  let client_id = env::var("AUTH0_CLIENT_ID")
    .expect("Unable to get AUTH0_CLIENT_ID env var.");


  let client_secret = env::var("AUTH0_CLIENT_SECRET")
    .expect("Unable to get AUTH0_CLIENT_SECRET env var.");
  
  let audience = env::var("AUTH0_API_ENDPOINT")
    .expect("AUTH0_AUDIENCE env var not found.");
  // let audience = env::var("AUTH0_AUDIENCE")
  //   .expect("AUTH0_AUDIENCE env var not found.");

  let token_endpoint = env::var("AUTH0_TOKEN_ENDPOINT")
    .expect("AUTH0_TOKEN_ENDPOINT env var not found");

  let params = [
    ("grant_type", "client_credentials"),
    ("client_id", &client_id),
    ("client_secret", &client_secret),
    ("audience", &audience)
  ];

  let auth0_token_response = req::new()
    .post(&token_endpoint)
    .form(&params)
    .send()?;

  handle_auth0_response::<Auth0TokenResponse>(auth0_token_response)
}

pub fn get_auth0_user(auth0id: &str) -> Result<Auth0User, Auth0RequestError> {
// pub fn get_auth0_user(auth0id: &str) -> Result<Auth0User, reqwest::Error> {
  let token = get_auth0_token()?;
  println!("access_token?: {:?}", token);

  let auth_string = format!("{} {}", token.token_type, token.access_token);


  let auth0_path = env::var("AUTH0_API_ENDPOINT")
    .expect("AUTH0_API_ENDPOINT env var not found.");

  let url = format!("{}users/{}", auth0_path, utf8_percent_encode(&auth0id, NON_ALPHANUMERIC).to_string());

  println!("url is {}", &url);

  let auth_user_response = req::new()
    .get(&url)
    .header("Authorization", auth_string)
    .send()?;

  handle_auth0_response(auth_user_response)
}

pub fn get_auth0_user_by_email(email: &str) -> Result<Auth0User, Auth0RequestError> {
  let token = get_auth0_token()?;

  let auth_string = format!("{} {}", token.token_type, token.access_token);

  let auth0_path = env::var("AUTH0_API_ENDPOINT")
    .expect("AUTH0_API_ENDPOINT env var not found.");

  // https://torus-rocks.auth0.com/api/v2/users-by-email?email=broch%40torus.rocks
  let url = format!("{}users-by-email?email={}", auth0_path, utf8_percent_encode(&email, NON_ALPHANUMERIC).to_string());

  let auth0_user_response = req::new()
    .get(&url)
    .header("Authorization", auth_string)
    .send()?;

  handle_auth0_response(auth0_user_response)
}