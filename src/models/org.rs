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

#[derive(Serialize, Deserialize, JSONPayload)]
pub struct NewOrg {
  pub name: String,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "orgs"]
pub struct IsertableNewOrg {
  pub name: String,
  pub created_by: Uuid,
  pub updated_by: Uuid,
}