use crate::{
  graphql::schema::{
    block::{BlockTypes, NewBlock, UpdateBlock},
    blocks::{
      basic_table_block::BasicTableBlock, owner_text_block::OwnerTextBlock,
      vendor_text_block::VendorTextBlock,
    },
  },
  services::db::cell_service::create_cell,
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use serde_json::json;
use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

use super::{
  cell_service::{DBCell, DBNewCell},
  dimension_service::{create_dimension, DBDimension, DBNewDimension},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DBBlock {
  pub id: Uuid,

  #[serde(rename = "blockType")]
  pub block_type: String,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  #[serde(rename = "portalViewId")]
  pub portal_view_id: Uuid,

  pub egress: String,

  pub block_data: serde_json::Value,

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
pub struct DBNewBlock {
  pub block_type: String,

  pub portal_id: Uuid,

  pub portal_view_id: Uuid,

  pub egress: String,

  pub block_data: serde_json::Value,
}

impl From<NewBlock> for DBNewBlock {
  fn from(new_block: NewBlock) -> Self {
    DBNewBlock {
      block_type: new_block
        .block_type
        .to_string(),
      portal_id: new_block.portal_id,
      portal_view_id: new_block.portal_view_id,
      egress: new_block.egress,
      block_data: new_block.block_data,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBUpdateBlock {
  pub id: Uuid,

  pub block_type: BlockTypes,

  pub block_data: Option<serde_json::Value>,
}

impl From<UpdateBlock> for DBUpdateBlock {
  fn from(update_block: UpdateBlock) -> Self {
    let block_data = update_block
      .block_data
      .clone()
      .map(|bc| match &update_block.block_type {
        BlockTypes::BasicTable => {
          let block: BasicTableBlock =
            serde_json::from_str(&bc).expect("Unable to parse BasicTableBlock data");
          serde_json::to_value(block)
            .expect("Unable to convert BasicTableBlock back to serde_json::Value")
        }
        BlockTypes::OwnerText => {
          let block: OwnerTextBlock =
            serde_json::from_str(&bc).expect("Unable to parse OwnerTextBlock data");
          serde_json::to_value(block)
            .expect("Unable to convert OwnerTextBlock back to serde_json::Value")
        }
        BlockTypes::VendorText => {
          println!("in heerererer.");
          let block: VendorTextBlock =
            serde_json::from_str(&bc).expect("Unable to parse VendorTextBlock data");
          serde_json::to_value(block)
            .expect("Unable to convert VendorTextBlock back to serde_json::Value")
        }
      });

    dbg!(&block_data);

    DBUpdateBlock {
      id: update_block.id,
      block_type: update_block.block_type,
      block_data,
    }
  }
}

pub struct DBBlockParts {
  pub blocks: Vec<DBBlock>,

  pub dimensions: Vec<DBDimension>,

  pub cells: Vec<DBCell>,
}

pub async fn get_block<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  block_id: Uuid,
) -> Result<DBBlock> {
  sqlx::query_as!(DBBlock, "select * from blocks where id  = $1", block_id)
    .fetch_one(pool)
    .await
    .map_err(anyhow::Error::from)
}

pub async fn get_blocks<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  portal_id: Uuid,
) -> Result<Vec<DBBlock>> {
  sqlx::query_as!(
    DBBlock,
    "select * from blocks where portal_id = $1",
    portal_id
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_block<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
  new_block: DBNewBlock,
) -> Result<DBBlock> {
  sqlx::query_as!(
    DBBlock,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into blocks (block_type, portal_id, portal_view_id, egress, block_data, created_by, updated_by)
    values ($2, $3, $4, $5, $6, (select id from _user), (select id from _user))
    returning *;
    "#,
    auth0_user_id,
    new_block.block_type,
    new_block.portal_id,
    new_block.portal_view_id,
    new_block.egress,
    new_block.block_data,
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn update_block<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
  updated_block: DBUpdateBlock,
) -> Result<DBBlock> {
  // dbg!(&updated_block);

  sqlx::query_as!(
    DBBlock,
    r#"
    with _user as (select * from users where auth0id = $1)
    update blocks
      set
        block_data = coalesce($3, block_data),
        updated_by = (select id from _user)
      where id = $2
      returning *;
    "#,
    auth0_user_id,
    updated_block.id,
    updated_block.block_data
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn delete_block<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  block_id: Uuid,
) -> Result<i32> {
  sqlx::query!("delete from blocks where id = $1", block_id)
    .execute(pool)
    .await
    .map(|qr| qr.rows_affected() as i32)
    .map_err(anyhow::Error::from)
}

pub async fn delete_blocks<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  block_ids: Vec<Uuid>,
) -> Result<i32> {
  sqlx::query!("delete from blocks where id = any($1)", &block_ids)
    .execute(pool)
    .await
    .map(|qr| qr.rows_affected() as i32)
    .map_err(anyhow::Error::from)
}

pub async fn create_owner_text_block(
  pool: PgPool,
  auth0_id: &str,
  portal_id: Uuid,
  portal_view_id: Uuid,
) -> Result<DBBlockParts> {
  let mut tx = pool.begin().await?;

  // Create Dimension
  let new_dim = DBNewDimension {
    portal_id: portal_id,
    name: format!("owner_text_block_{}", Uuid::new_v4()),
    dimension_type: String::from("OwnerText"), // TODO: Probably should have an enum of dimension types.
    dimension_data: serde_json::Value::Null,   // TODO: Propagate this all the way to graphql
  };

  let db_dimension = create_dimension(&mut tx, auth0_id, new_dim).await?;

  // Create Cell
  let new_cell = DBNewCell {
    portal_id,
    dimensions: vec![db_dimension.id],
    cell_type: String::from("OwnerText"), // TODO: Figure types for cells out.
    cell_data: json!({
      "text": "Little bit of starting text..."
    }),
  };

  let db_cell = create_cell(&mut tx, auth0_id, new_cell).await?;

  // Create Block
  let new_block = DBNewBlock {
    block_type: String::from("OwnerText"),
    portal_id,
    portal_view_id,
    egress: String::from("owner"),
    block_data: serde_json::to_value(OwnerTextBlock {
      content_dimension_id: Some(
        db_dimension
          .id
          .clone(),
      ),
    })?,
  };

  let db_block = create_block(&mut tx, auth0_id, new_block).await?;

  tx.commit().await?;

  Ok(DBBlockParts {
    blocks: vec![db_block],
    dimensions: vec![db_dimension],
    cells: vec![db_cell],
  })
}

pub async fn create_vendor_text_block(
  pool: PgPool,
  auth0_id: &str,
  portal_id: Uuid,
  portal_view_id: Uuid,
) -> Result<DBBlock> {
  let mut tx = pool.begin().await?;

  // Create Block
  let new_block = DBNewBlock {
    block_type: String::from("VendorText"),
    portal_id,
    portal_view_id,
    egress: String::from("vendor"),
    block_data: serde_json::to_value(VendorTextBlock {
      content_dimension_id: None,
    })?,
  };

  let db_block = create_block(&mut tx, auth0_id, new_block).await?;

  tx.commit().await?;

  Ok(db_block)
}
