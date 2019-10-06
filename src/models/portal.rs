use crate::schema::portals;
use crate::futures::{ Future, future::ok as fut_ok };
use actix_web::{ FromRequest, HttpRequest, error, dev };
use chrono::naive::NaiveDateTime;
use uuid::Uuid;

#[derive(Serialize, Queryable)]
pub struct Portal {
  pub id: Uuid,
  pub name: String,
  pub org: Uuid,
  pub owners: Vec<Uuid>,
  pub vendors: Vec<Uuid>,
  pub created_at: NaiveDateTime,
  pub created_by: Uuid,
  pub updated_at: NaiveDateTime,
  pub updated_by: Uuid,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "portals"]
pub struct NewPortal {
  pub org: Uuid,
  pub name: String,
  pub created_by: Uuid,
  pub updated_by: Uuid,
  pub owners: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, JSONPayload)]
pub struct NewPortalPayload {
  pub org: Uuid,
  pub name: String,
}