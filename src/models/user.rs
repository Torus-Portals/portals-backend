use crate::schema::users;
use crate::futures::{ Future };
use actix_web::{ FromRequest, HttpRequest, error, dev };

#[derive(Serialize, Queryable)]
pub struct User {
  pub id: i32,
  pub username: String,
  pub firstname: String,
  pub lastname: String,
  pub email: String,
  pub orgs: Vec<i32>,
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

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[table_name = "users"]
pub struct UpdateUser {
  pub username: Option<String>,
  pub firstname: Option<String>,
  pub lastname: Option<String>,
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