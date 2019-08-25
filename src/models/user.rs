use serde_json;
use crate::schema::users;
use crate::futures::{ Future, Stream };
use actix_web::{ FromRequest, HttpRequest, error, dev, web };

#[derive(Serialize, Queryable)]
pub struct User {
  pub id: i32,
  pub username: String,
  pub firstname: String,
  pub lastname: String,
  pub email: String,
  pub email_confirmed: bool,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
  pub username: String,
  pub firstname: String,
  pub lastname: String,
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