use super::DB;

use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DBPortal {
  pub id: Uuid,

  pub name: String,

  pub org: Uuid,

  pub owner_ids: Vec<Uuid>,

  pub vendor_ids: Vec<Uuid>,

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

  pub owner_ids: Vec<Uuid>,

  pub vendor_ids: Vec<Uuid>,
}

impl DB {
  pub async fn get_portal(&self, portal_id: Uuid) -> Result<DBPortal> {
    sqlx::query_as!(DBPortal, "select * from portals where id = $1", portal_id)
      .fetch_one(&self.pool)
      .await
      .map_err(anyhow::Error::from)
  }

  pub async fn get_portals(&self, portal_ids: Vec<Uuid>) -> Result<Vec<DBPortal>> {
    sqlx::query_as!(
      DBPortal,
      "select * from portals where id = any($1)",
      &portal_ids
    )
    .fetch_all(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }

  pub async fn get_auth0_user_portals(&self, auth0_user_id: &str) -> Result<Vec<DBPortal>> {
    sqlx::query_as!(
      DBPortal,
      r#"
      with _user as (select * from users where auth0id = $1)
      select * from portals where
      (select id from _user) = any(owner_ids) or
      (select id from _user) = any(vendor_ids);
      "#,
      auth0_user_id
    )
    .fetch_all(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }
}
