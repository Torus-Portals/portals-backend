use reqwest;

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
  let params = [
    ("grant_type", "client_credentials"),
    ("client_id", "A1OENRL6JWt44fx8z4Oc6hD875N5mQ0S"),
    ("client_secret", "kKBEs6WM_36CAHBmA4tYiUAhScExBv1gF0WSznosqignhLR1IMN3Kyal7updZg89"),
    ("audience", "https://torus-rocks.auth0.com/api/v2/")
  ];

  let auth0_token_response: Auth0TokenResponse = reqwest::Client::new()
    .post("https://torus-rocks.auth0.com/oauth/token")
    .form(&params)
    .send()?
    .json()?;

  Ok(auth0_token_response)
}

pub fn get_auth0_user(auth0id: &str) -> Result<Auth0User, reqwest::Error> {
  let token = get_auth0_token()?;

  let auth_string = format!("{} {}", token.token_type, token.access_token);

  let auth0_path = String::from("https://torus-rocks.auth0.com/api/v2/users/");

  let url = format!("{}{}", auth0_path, utf8_percent_encode(&auth0id, NON_ALPHANUMERIC).to_string());

  let auth0_user: Auth0User = reqwest::Client::new()
    .get(&url)
    .header("Authorization", auth_string)
    .send()?
    .json()?;

  Ok(auth0_user)
}