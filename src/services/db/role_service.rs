use crate::graphql::schema::role::NewRole;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DBRole {
  pub id: Uuid,

  pub role_type: String,

  pub perms: serde_json::Value,

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
pub struct DBNewRole {
  pub role_type: String,
}

impl From<NewRole> for DBNewRole {
  fn from(new_role: NewRole) -> Self {
    DBNewRole {
      role_type: new_role
        .role_type
        .to_string(),
    }
  }
}

pub async fn get_role<'e>(pool: impl Executor<'e, Database = Postgres>, role_id: Uuid) -> Result<DBRole> {
  sqlx::query_as!(DBRole, "select * from roles where id = $1", role_id)
    .fetch_one(pool)
    .await
    .map_err(anyhow::Error::from)
}

pub async fn create_role<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0id: &str,
  new_role: DBNewRole,
) -> Result<DBRole> {
  sqlx::query_as!(
      DBRole,
      r#"
      with _user as (select * from users where auth0id = $1)
      insert into roles (role_type, created_by, updated_by) values ($2, (select id from _user), (select id from _user))
      returning *
      "#,
      auth0id,
      new_role.role_type
    )
    .fetch_one(pool)
    .await
    .map_err(anyhow::Error::from)
}
