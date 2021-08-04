use std::any;

use super::DB;

use anyhow::Result;
use chrono::{DateTime, Utc};

use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct DBPortalView {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  pub egress: String,

  pub access: String,

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
  pub async fn get_portal_views(&self, portal_id: Uuid) -> Result<Vec<DBPortalView>> {
    sqlx::query_as!(
      DBPortalView,
      r#"
      select * from portalviews where portal_id = $1
      "#,
      portal_id
    )
    .fetch_all(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }
}
