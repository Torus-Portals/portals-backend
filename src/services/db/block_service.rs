use std::str::FromStr;

use crate::graphql::schema::{
  block::{BlockTypes, NewBlock, UpdateBlock},
  blocks::table_block::TableBlock,
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DBBlock {
  pub id: Uuid,

  pub name: String,

  pub project_id: Uuid,

  pub dashboard_id: Uuid,

  pub page_id: Uuid,

  pub block_type: String,

  pub block_data: serde_json::Value,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,

  pub deleted_at: Option<DateTime<Utc>>,

  pub deleted_by: Option<Uuid>,
}

impl From<DBBlock> for DBUpdateBlock {
  fn from(db_block: DBBlock) -> Self {
    DBUpdateBlock {
      id: db_block.id,
      name: Some(db_block.name),
      block_type: BlockTypes::from_str(&db_block.block_type)
        .expect("Unable to convert block_type string to BlockTypes Enum"),
      block_data: Some(db_block.block_data),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBNewBlock {
  pub project_id: Uuid,

  pub dashboard_id: Uuid,

  pub page_id: Uuid,

  pub block_type: String,

  pub block_data: serde_json::Value,
}

impl From<NewBlock> for DBNewBlock {
  fn from(new_block: NewBlock) -> Self {
    let block_data = block_string_to_serde_value(&new_block.block_type, new_block.block_data)
      .expect("unable to convert block data into serde_json::Value");

    DBNewBlock {
      project_id: new_block.project_id,
      dashboard_id: new_block.dashboard_id,
      page_id: new_block.page_id,
      block_type: new_block
        .block_type
        .to_string(),
      block_data,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBUpdateBlock {
  pub id: Uuid,

  pub name: Option<String>,

  pub block_type: BlockTypes,

  pub block_data: Option<serde_json::Value>,
}

impl From<UpdateBlock> for DBUpdateBlock {
  fn from(update_block: UpdateBlock) -> Self {
    let block_data = update_block
      .block_data
      .clone()
      .map(|bd| {
        block_string_to_serde_value(&update_block.block_type, bd)
          .expect("unable to convert block data to serde_json::Value")
      });

    DBUpdateBlock {
      id: update_block.id,
      name: update_block.name,
      block_type: update_block.block_type,
      block_data,
    }
  }
}

pub async fn get_block(
  pool: impl PgExecutor<'_>,
  block_id: Uuid,
) -> Result<DBBlock> {
  sqlx::query_as!(DBBlock, "select * from blocks where id = $1", block_id)
    .fetch_one(pool)
    .await
    .map_err(anyhow::Error::from)
}

pub async fn get_blocks(
  pool: impl PgExecutor<'_>,
  block_ids: Vec<Uuid>,
) -> Result<Vec<DBBlock>> {
  sqlx::query_as!(
    DBBlock,
    "select * from blocks where id = any($1)",
    &block_ids
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_page_blocks(
  pool: impl PgExecutor<'_>,
  page_id: Uuid,
) -> Result<Vec<DBBlock>> {
  sqlx::query_as!(
    DBBlock,
    "select * from blocks where page_id = $1",
    page_id
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_project_blocks(
  pool: impl PgExecutor<'_>,
  project_id: Uuid,
) -> Result<Vec<DBBlock>> {
  sqlx::query_as!(
    DBBlock,
    "select * from blocks where project_id = $1",
    project_id
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_block(
  pool: impl PgExecutor<'_>,
  auth0_user_id: &str,
  new_block: DBNewBlock,
) -> Result<DBBlock> {
  sqlx::query_as!(
    DBBlock,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into blocks (name, project_id, dashboard_id, page_id, block_type, block_data, created_by, updated_by)
    values ($2, $3, $4, $5, $6, $7, (select id from _user), (select id from _user))
    returning *;
    "#,
    auth0_user_id,
    format!("New {}", new_block.block_type), // replace this.
    new_block.project_id,
    new_block.dashboard_id,
    new_block.page_id,
    new_block.block_type,
    new_block.block_data
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn update_block(
  pool: impl PgExecutor<'_>,
  auth0_user_id: &str,
  updated_block: DBUpdateBlock,
) -> Result<DBBlock> {
  sqlx::query_as!(
    DBBlock,
    r#"
    with _user as (select * from users where auth0id = $1)
    update blocks
      set
        name = coalesce($3, name),
        block_data = coalesce($4, block_data),
        updated_by = (select id from _user)
      where id = $2
      returning *;
    "#,
    auth0_user_id,
    updated_block.id,
    updated_block.name,
    updated_block.block_data
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn delete_block(
  pool: impl PgExecutor<'_>,
  auth0_id: &str,
  block_id: Uuid,
) -> Result<DateTime<Utc>> {
  sqlx::query_as!(
    DBBlock,
    r#"
    with _user as (select * from users where auth0id = $1)
    update blocks
      set
        deleted_by = (select id from _user),
        deleted_at = now()
      where id = $2
    returning *;
    "#,
    auth0_id,
    block_id
  )
  .fetch_one(pool)
  .await
  .map(|db_b| {
    db_b
      .deleted_at
      .unwrap()
  })
  .map_err(anyhow::Error::from)
}

pub async fn delete_blocks(
  pool: impl PgExecutor<'_>,
  block_ids: Vec<Uuid>,
) -> Result<i32> {
  sqlx::query!("delete from blocks where id = any($1)", &block_ids)
    .execute(pool)
    .await
    .map(|qr| qr.rows_affected() as i32)
    .map_err(anyhow::Error::from)
}

pub async fn delete_page_blocks(
  pool: impl PgExecutor<'_>,
  page_id: Uuid,
) -> Result<i32> {
  sqlx::query!("delete from blocks where page_id = $1", page_id)
    .execute(pool)
    .await
    .map(|qr| qr.rows_affected() as i32)
    .map_err(anyhow::Error::from)
}

pub fn block_string_to_serde_value(
  block_type: &BlockTypes,
  bd: String,
) -> Result<serde_json::Value> {
  let value = match block_type {
    BlockTypes::Table => {
      let block: TableBlock = serde_json::from_str(&bd)?;
      serde_json::to_value(block)
    }
  };

  value.map_err(anyhow::Error::from)
}
