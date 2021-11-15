use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::graphql::schema::connection::NewConnection;

#[derive(Debug, Clone)]
pub struct DBConnection {
  pub id: Uuid,

  pub block_id: Uuid,

  pub source_id: Option<Uuid>,

  pub sourcequery_id: Option<Uuid>,

  pub destination_id: Option<Uuid>,

  pub destination_type: Option<String>,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

pub struct DBNewConnection {
  pub block_id: Uuid,

  pub source_id: Option<Uuid>,

  pub sourcequery_id: Option<Uuid>,

  pub destination_id: Option<Uuid>,

  pub destination_type: Option<String>,
}

impl From<NewConnection> for DBNewConnection {
  fn from(new_connection: NewConnection) -> Self {
    let destination_type = new_connection.destination_type.map(|destination_type| {
      destination_type.to_string()
    });

    Self {
      block_id: new_connection.block_id,
      source_id: new_connection.source_id,
      sourcequery_id: new_connection.query_id,
      destination_id: new_connection.destination_id,
      destination_type,
    }
  }
}

pub async fn get_connections(
  pool: impl PgExecutor<'_>,
  block_id: Uuid,
) -> Result<Vec<DBConnection>> {
  sqlx::query_as!(
    DBConnection,
    "select * from connections where block_id = $1",
    block_id
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_connection(pool: impl PgExecutor<'_>, auth0_id: &str, connection: DBNewConnection) -> Result<DBConnection> {
  sqlx::query_as!(
    DBConnection,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into connections (
      block_id,
      source_id,
      sourcequery_id,
      destination_id,
      destination_type,
      created_by,
      updated_by
    ) values (
      $2,
      $3,
      $4,
      $5,
      $6,
      (select id from _user),
      (select id from _user)
    )
    returning *;
    "#,
    auth0_id,
    connection.block_id,
    connection.source_id,
    connection.sourcequery_id,
    connection.destination_id,
    connection.destination_type
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}
