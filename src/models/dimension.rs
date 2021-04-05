use actix_web::{dev, error, FromRequest, HttpRequest};
use chrono::naive::NaiveDateTime;
use serde_json;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Dimension {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  #[serde(rename = "dimensionType")]
  pub dimension_type: String,

  pub meta: serde_json::Value,

  #[serde(rename = "createdAt")]
  pub created_at: NaiveDateTime,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: NaiveDateTime,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct NewDimension {
  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  #[serde(rename = "dimensionType")]
  pub dimension_type: String,

  pub meta: serde_json::Value,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[derive(Serialize, Deserialize, JSONPayload)]
pub struct NewDimensionPayload {
  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  #[serde(rename = "dimensionType")]
  pub dimension_type: String,

  pub meta: serde_json::Value,
}

#[derive(Serialize, Deserialize, JSONPayload)]
pub struct NewDimensionsPayload(pub Vec<NewDimensionPayload>);
