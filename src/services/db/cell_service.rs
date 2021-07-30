use super::DB;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DBCell {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  // NOTE: Should probably be good to make this an enum.
  #[serde(rename = "cellType")]
  pub cell_type: String,

  pub dimensions: Vec<Uuid>,

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

impl DB {
  pub async fn get_cell(&self, cell_id: Uuid) -> Result<DBCell> {
    sqlx::query_as!(DBCell, "select * from cells where id = $1", cell_id)
      .fetch_one(&self.pool)
      .await
      .map_err(anyhow::Error::from)
  }
}