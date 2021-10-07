use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::graphql::schema::integration::{GoogleSheetsIntegration, NewIntegration};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DBIntegration {
  pub id: Uuid,

  pub name: String,

  pub portal_id: Uuid,

  pub integration_type: String,

  pub integration_data: serde_json::Value,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DBNewIntegration {
  pub portal_id: Uuid,

  pub name: String,

  pub integration_type: String,
  // TODO: replace with json string to insert into Postgres jsonb format
  // JSON response from API call
  // pub integration_data: serde_json::Value,
}

impl From<NewIntegration> for DBNewIntegration {
  fn from(new_integration: NewIntegration) -> Self {
    DBNewIntegration {
      portal_id: new_integration.portal_id,
      name: new_integration.name,
      integration_type: new_integration.integration_type.to_string(),
    }
  }
}

pub async fn get_integration<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  integration_id: Uuid,
) -> Result<DBIntegration> {
  sqlx::query_as!(
    DBIntegration,
    "select * from integrations where id = $1",
    integration_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_integrations<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  portal_id: Uuid,
) -> Result<Vec<DBIntegration>> {
  sqlx::query_as!(
    DBIntegration,
    "select * from integrations where portal_id = $1",
    portal_id
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_integration<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
  new_integration: DBNewIntegration,
) -> Result<DBIntegration> {
  sqlx::query_as!(
    DBIntegration,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into integrations (name, portal_id, integration_type, created_by, updated_by)
    values ($2, $3, $4, (select id from _user), (select id from _user))
    returning *;
    "#,
    auth0_user_id,
    new_integration.name,
    new_integration.portal_id,
    new_integration.integration_type,
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

// pub async fn create_google_sheets_integration<'e>(
//   pool: impl Executor<'e, Database = Postgres>,
//   auth0_user_id: &str,
//   new_integration: DBNewIntegration,
// ) -> Result<DBIntegration> {
// }

pub async fn delete_integration<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  integration_id: Uuid,
) -> Result<i32> {
  sqlx::query!("delete from integrations where id = $1", integration_id)
    .execute(pool)
    .await
    .map(|qr| qr.rows_affected() as i32)
    .map_err(anyhow::Error::from)
}
