use crate::schema::orgs;
use actix_web::{ FromRequest, HttpRequest, error, dev };
use chrono::naive::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct Org {
  pub id: Uuid,

  pub name: String,

  #[serde(rename = "createdAt")]
  pub created_at: NaiveDateTime,
  
  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: NaiveDateTime,

  #[serde(rename = "updatedBy")]
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

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedBy")]  
  pub updated_by: Uuid,
}