use crate::schema::users;
use futures::future::{ ok, Ready};
use actix_web::{ FromRequest, HttpRequest, error, dev };
use chrono::naive::NaiveDateTime;
use uuid::Uuid;

use jwt::dangerous_unsafe_decode;

#[derive(Debug, Serialize, Queryable)]
pub struct User {
  pub id: Uuid,

  pub auth0id: String,

  pub name: String,

  pub nickname: String,

  pub email: String,

  // TODO: Maybe try to figure out how to use postgres enums with status.
  pub status: String, 

  pub orgs: Vec<Uuid>,

  #[serde(rename = "createdAt")]
  pub created_at: NaiveDateTime,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: NaiveDateTime,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Insertable, JSONPayload)]
#[table_name = "users"]
pub struct NewUser {
  pub auth0id: String,

  pub name: String,

  pub nickname: String,

  pub email: String,

  pub status: String,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset, JSONPayload)]
#[table_name = "users"]
pub struct UpdateUser {
  pub auth0id: Option<String>,

  pub name: Option<String>,

  pub nickname: Option<String>,

  pub email: Option<String>,

  pub status: Option<String>,

  pub orgs: Option<Vec<Uuid>>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Option<Uuid>,
}

#[derive(Serialize, Deserialize, JSONPayload)]
pub struct InvitedUser {
  pub email: String,
  #[serde(rename = "portalId")]
  pub portal_id: Uuid,
  pub egress: String,
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
  type Future = Ready<Result<Self, Self::Error>>;
  type Config = ();

  fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
    let access_token_header_val = req.headers().get("authorization").unwrap();
    let access_token_str = access_token_header_val.to_str().unwrap();
    let access_token: Vec<&str> = access_token_str.split_whitespace().collect();

    // Okay do dangerous_unsafe_decode here because the user has already verified in middleware.
    let decoded_token = dangerous_unsafe_decode::<Auth0UserClaims>(access_token.get(1).unwrap()).ok().unwrap();

    ok(Auth0UserId { id: decoded_token.claims.sub.clone() })
  }
}