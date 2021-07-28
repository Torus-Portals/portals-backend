use super::DB;

use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DBPortal {
  pub id: Uuid,

  pub name: String,

  pub org: Uuid,

  pub owners: Vec<Uuid>,

  pub vendors: Vec<Uuid>,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct DBNewPortal {
  pub org: Uuid,

  pub name: String,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,

  pub owners: Vec<Uuid>,
}

impl DB {
  pub async fn get_portal(&self, portal_id: Uuid) -> Result<DBPortal> {
    sqlx::query_as!(DBPortal, "select * from portals where id = $1", portal_id)
      .fetch_one(&self.pool)
      .await
      .map_err(anyhow::Error::from)
  }
}
