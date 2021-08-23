use super::DB;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DBStructure {
  pub id: Uuid,

  pub structure_type: String,

  pub structure_data: serde_json::Value,

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
pub struct DBNewStructure {
  pub structure_type: String,

  pub structure_data: serde_json::Value,
}

impl DB {
  pub async fn get_structure(&self, structure_id: Uuid) -> Result<DBStructure> {
    sqlx::query_as!(
      DBStructure,
      "select * from structures where id = $1",
      structure_id
    )
    .fetch_one(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }

  pub async fn get_structures(&self, ids: &[Uuid]) -> Result<Vec<DBStructure>> {
    sqlx::query_as!(
      DBStructure,
      "select * from structures where id = any($1)",
      ids
    )
    .fetch_all(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }

  pub async fn create_structure(
    &self,
    auth0_user_id: &str,
    new_structure: DBNewStructure,
  ) -> Result<DBStructure> {
    sqlx::query_as!(
      DBStructure,
      r#"
      with _user as (select * from users where auth0id = $1)
      insert into structures (
        structure_type,
        structure_data,
        created_by,
        updated_by
      ) values (
        $2,
        $3,
        (select id from _user),
        (select id from _user)
      ) returning *
      "#,
      auth0_user_id,
      new_structure.structure_type,
      new_structure.structure_data,
    )
    .fetch_one(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }
}
