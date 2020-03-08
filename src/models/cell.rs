use crate::schema::cells;
use actix_web::{ FromRequest, HttpRequest, error, dev };
use chrono::naive::NaiveDateTime;
use uuid::Uuid;
use serde_json;

#[derive(Serialize, Queryable)]
pub struct Cell {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  // NOTE: Should probably be good to make this an enum.
  #[serde(rename = "cellType")]
  pub cell_type: String,

  pub dimensions: Vec<Uuid>,

  pub data: serde_json::Value,

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
#[table_name = "cells"]
pub struct NewCell {
  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  #[serde(rename = "cellType")]
  pub cell_type: String,

  pub dimensions: Vec<Uuid>,

  pub data: serde_json::Value,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[derive(Serialize, Deserialize, JSONPayload)]
pub struct NewCellPayload {
  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  #[serde(rename = "cellType")]
  pub cell_type: String,

  pub dimensions: Vec<Uuid>,

  pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, JSONPayload)]
pub struct NewCellsPayload (pub Vec<NewCellPayload>);

#[derive(Debug, Serialize, Deserialize, AsChangeset, JSONPayload)]
#[table_name = "cells"]
pub struct UpdateCell {
  #[serde(rename = "cellType")]
  pub cell_type: Option<String>,

  pub dimensions: Option<Vec<Uuid>>,

  pub data: Option<serde_json::Value>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Option<Uuid>,
}