use super::DB;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use uuid::Uuid;


#[derive(Debug, Serialize)]
pub struct DBUser {
  pub id: Uuid,

  pub auth0id: String,

  pub name: String,

  pub nickname: String,

  pub email: String,

  // TODO: Maybe try to figure out how to use postgres enums with status.
  pub status: String,

  pub orgs: Vec<Uuid>,

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
  pub async fn get_user(&self, user_id: Uuid) -> Result<DBUser> {
    sqlx::query_as!(DBUser, "select * from users where id = $1", user_id)
    .fetch_one(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }
}