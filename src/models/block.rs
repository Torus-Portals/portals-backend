use crate::schema::blocks;
use crate::futures::{ Future, future::ok as fut_ok };
use actix_web::{ FromRequest, HttpRequest, error, dev };
use chrono::naive::NaiveDateTime;
use uuid::Uuid;
use serde_json;

#[derive(Serialize, Queryable)]
pub struct Block {
  pub id: Uuid,
  pub block_type: String,
  pub portal_id: Uuid,
  pub portal_view_id: Uuid,
  pub egress: String,
  pub bbox: Vec<i32>,
  pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, JSONPayload)]
pub struct NewBlockPayload {
  pub block_type: String,
  pub portal_id: Uuid,
  pub portal_view_id: Uuid,
  pub egress: String,
  pub bbox: Vec<i32>,
  pub data: serde_json::Value,
}