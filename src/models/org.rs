use crate::schema::orgs;
use crate::futures::{ Future };
use actix_web::{ FromRequest, HttpRequest, error, dev };

#[derive(Serialize, Queryable)]
pub struct Org {
  pub id: i32,
  pub name: String,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "orgs"]
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