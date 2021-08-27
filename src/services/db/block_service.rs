use crate::graphql::schema::block::NewBlock;

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

  pub block_data: serde_json::Value,

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

  pub block_data: serde_json::Value,
}

impl From<NewBlock> for DBNewBlock {
  fn from(new_block: NewBlock) -> Self {
    let block_data = serde_json::from_str(&new_block.block_data).expect("block data isn't json, dude");

    DBNewBlock {
        block_type: new_block.block_type.to_string(),
        portal_id: new_block.portal_id,
        portal_view_id: new_block.portal_view_id,
        egress: new_block.egress,
        block_data: block_data,
    }
  }
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

  pub async fn create_block(&self, auth0_user_id: &str, new_block: DBNewBlock) -> Result<DBBlock> {
    sqlx::query_as!(
      DBBlock,
      r#"
      with _user as (select * from users where auth0id = $1)
      insert into blocks (block_type, portal_id, portal_view_id, egress, block_data, created_by, updated_by)
      values ($2, $3, $4, $5, $6, (select id from _user), (select id from _user))
      returning *;
      "#,
      auth0_user_id,
      new_block.block_type,
      new_block.portal_id,
      new_block.portal_view_id,
      new_block.egress,
      new_block.block_data,
    )
    .fetch_one(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }
}
