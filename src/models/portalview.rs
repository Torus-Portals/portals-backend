use actix_web::{ FromRequest, HttpRequest, error, dev };
use chrono::naive::NaiveDateTime;
use uuid::Uuid;

#[derive(Serialize)]
pub struct PortalView {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  pub egress: String,

  pub access: String,

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
pub struct NewPortalView {
  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  pub egress: String,

  pub access: String,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,
  
  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[derive(Serialize, Deserialize, JSONPayload)]
pub struct NewPortalViewPayload {
  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  pub egress: String,

  pub access: String,
}