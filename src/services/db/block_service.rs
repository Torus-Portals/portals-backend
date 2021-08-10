use super::DB;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DBBlock {
  pub id: Uuid,

  #[serde(rename = "blockType")]
  pub block_type: String,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  #[serde(rename = "portalViewId")]
  pub portal_view_id: Uuid,

  pub egress: String,

  pub bbox: Vec<i32>,

  pub data: serde_json::Value,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBNewBlock {
  pub block_type: String,

  pub portal_id: Uuid,

  pub portal_view_id: Uuid,

  pub egress: String,

  pub bbox: Vec<i32>,

  pub data: serde_json::Value,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

impl DB {
  pub async fn get_block(&self, block_id: Uuid) -> Result<DBBlock> {
    sqlx::query_as!(DBBlock, "select * from blocks where id  = $1", block_id)
      .fetch_one(&self.pool)
      .await
      .map_err(anyhow::Error::from)
  }

  pub async fn get_blocks(&self, portal_id: Uuid) -> Result<Vec<DBBlock>> {
    sqlx::query_as!(
      DBBlock,
      "select * from blocks where portal_id = $1",
      portal_id
    )
    .fetch_all(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }
}
