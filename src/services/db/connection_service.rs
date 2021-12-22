use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;
use std::convert::TryInto;

use crate::graphql::schema::{connection::NewConnection, sources::block_source::BlockSource, block::Block};

use super::{
  block_service::{get_block},
  source_service::{get_source, DBSource},
  sourcequery_service::{get_sourcequery, DBSourceQuery},
};

#[derive(Debug, Clone)]
pub struct DBConnection {
  pub id: Uuid,

  pub name: String,

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
  pub name: String,

  pub block_id: Uuid,

  pub source_id: Option<Uuid>,

  pub sourcequery_id: Option<Uuid>,

  pub destination_id: Option<Uuid>,

  pub destination_type: Option<String>,
}

impl From<NewConnection> for DBNewConnection {
  fn from(new_connection: NewConnection) -> Self {
    let destination_type = new_connection
      .destination_type
      .map(|destination_type| destination_type.to_string());

    Self {
      name: new_connection.name,
      block_id: new_connection.block_id,
      source_id: new_connection.source_id,
      sourcequery_id: new_connection.sourcequery_id,
      destination_id: new_connection.destination_id,
      destination_type,
    }
  }
}

pub async fn get_connection(
  pool: impl PgExecutor<'_>,
  connection_id: Uuid,
) -> Result<DBConnection> {
  sqlx::query_as!(
    DBConnection,
    r#"
    select * from connections where id = $1
    "#,
    connection_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
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

pub async fn create_connection(
  pool: impl PgExecutor<'_>,
  auth0_id: &str,
  connection: DBNewConnection,
) -> Result<DBConnection> {
  sqlx::query_as!(
    DBConnection,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into connections (
      name,
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
      $7,
      (select id from _user),
      (select id from _user)
    )
    returning *;
    "#,
    auth0_id,
    connection.name,
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

pub struct ConnectionData {
  pub connection: DBConnection,

  pub source: Option<DBSource>,

  pub source_block: Option<Block>,

  pub sourcequery: Option<DBSourceQuery>,

  pub destination_block: Option<Block>,
}

pub async fn get_connection_data(pool: PgPool, connection_id: Uuid) -> Result<ConnectionData> {
  let mut tx = pool.begin().await?;

  let connection = get_connection(&mut tx, connection_id).await?;

  let source = match connection.source_id {
    Some(source_id) => get_source(&mut tx, source_id)
      .await
      .ok(),
    None => None,
  };

  let source_block = match &source {
    Some(s) => {
      let bs = serde_json::from_value::<BlockSource>(
        s.source_data
          .to_owned(),
      )?;
      let db_block = get_block(&mut tx, bs.block_id).await.ok();

      let block = match db_block {
        Some(b) => {
          let b: Block = b.try_into()?;
          Some(b)
        },
        None => None,
      };

      block
    }
    None => None,
  };

  let sourcequery = match connection.sourcequery_id {
    Some(sourcequery_id) => get_sourcequery(&mut tx, sourcequery_id)
      .await
      .ok(),
    None => None,
  };

  let destination_block = match connection.destination_id {
    Some(destination_id) => {
      let db_block = get_block(&mut tx, destination_id).await.ok();
      let block = match db_block {
        Some(b) => {
          let b: Block = b.try_into()?;
          Some(b)
        },
        None => None,
      };

      block
    },
    None => None,
  };

  tx.commit().await?;

  Ok(ConnectionData {
    connection,
    source,
    source_block,
    sourcequery,
    destination_block,
  })
}
