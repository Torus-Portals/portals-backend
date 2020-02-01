use crate::schema::dimensions;
use actix_web::{ FromRequest, HttpRequest, error, dev };
use chrono::naive::NaiveDateTime;
use uuid::Uuid;
use serde_json;

#[derive(Serialize, Queryable)]
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

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "dimensions"]
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
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  #[serde(rename = "dimensionType")]
  pub dimension_type: String,

  pub meta: serde_json::Value,
}

#[derive(Serialize, Deserialize, JSONPayload)]
pub struct NewDimensionsPayload (pub Vec<NewDimensionPayload>);
