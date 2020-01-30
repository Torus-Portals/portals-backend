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

  pub dimensions: Vec<Uuid>,

  pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, JSONPayload)]
pub struct NewCellsPayload (pub Vec<NewCellPayload>);