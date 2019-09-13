use crate::schema::orgs;
use crate::futures::{ Future };
use actix_web::{ FromRequest, HttpRequest, error, dev };
use chrono::naive::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct Org {
  pub id: Uuid,
  pub name: String,
  pub created_at: NaiveDateTime,
  pub created_by: Uuid,
  pub updated_at: NaiveDateTime,
  pub updated_by: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct NewOrg {
  pub name: String,
}

impl FromRequest for NewOrg {
  type Error = error::JsonPayloadError;
  type Future = Box<dyn Future<Item = Self, Error = error::JsonPayloadError>>;
  type Config = ();

  fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
    Box::new(
      dev::JsonBody::<Self>::new(req, payload, None)
    )
  }
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "orgs"]
pub struct IsertableNewOrg {
  pub name: String,
  pub created_by: Uuid,
  pub updated_by: Uuid,
}