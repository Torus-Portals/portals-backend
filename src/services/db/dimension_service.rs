use super::DB;

use anyhow::Result;
use chrono::{DateTime, Utc};

use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct DBDimension {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  #[serde(rename = "dimensionType")]
  pub dimension_type: String,

  pub meta: serde_json::Value,

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
  pub async fn get_dimensions(&self, portal_id: Uuid) -> Result<Vec<DBDimension>> {
    sqlx::query_as!(
      DBDimension,
      r#"select * from dimensions where portal_id = $1 "#,
      portal_id
    )
    .fetch_all(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }
}