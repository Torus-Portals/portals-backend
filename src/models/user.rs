use crate::schema::users;
use crate::futures::{ Future, future::ok as fut_ok };
use actix_web::{ FromRequest, HttpRequest, error, dev };

use jwt::dangerous_unsafe_decode;

#[derive(Serialize, Queryable)]
pub struct User {
  pub id: i32,
  pub auth0id: String,
  pub name: String,
  pub nickname: String,
  pub email: String,
  pub orgs: Vec<i32>,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
  pub auth0id: String,
  pub name: String,
  pub nickname: String,
  pub email: String,
}

impl FromRequest for NewUser {
  type Error = error::JsonPayloadError;
  type Future = Box<dyn Future<Item = Self, Error = error::JsonPayloadError>>;
  type Config = ();

  fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
    Box::new(
      dev::JsonBody::<Self>::new(req, payload, None)
    )
  }
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[table_name = "users"]
pub struct UpdateUser {
  pub auth0id: Option<String>,
  pub name: Option<String>,
  pub nickname: Option<String>,
  pub email: Option<String>,
  pub orgs: Option<Vec<i32>>
}

impl FromRequest for UpdateUser {
  type Error = error::JsonPayloadError;
  type Future = Box<dyn Future<Item = Self, Error = error::JsonPayloadError>>;
  type Config = ();

  fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
    Box::new(
      dev::JsonBody::<Self>::new(req, payload, None)
    )
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth0UserId {
  pub id: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth0UserClaims {
  pub sub: String
}

impl FromRequest for Auth0UserId {
  type Error = error::JsonPayloadError;
  type Future = Box<dyn Future<Item = Self, Error = error::JsonPayloadError>>;
  type Config = ();

  fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
    let access_token_header_val = req.headers().get("authorization").unwrap();
    let access_token_str = access_token_header_val.to_str().unwrap();
    let access_token: Vec<&str> = access_token_str.split_whitespace().collect();

    let decoded_token = dangerous_unsafe_decode::<Auth0UserClaims>(access_token.get(1).unwrap()).ok().unwrap();

    Box::new(
      fut_ok(Auth0UserId { id: decoded_token.claims.sub.clone() })
    )
  }
}