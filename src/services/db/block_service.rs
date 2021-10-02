use std::{collections::HashSet, str::FromStr};

use crate::{
  graphql::schema::{
    block::{BlockTypes, NewBlock, UpdateBlock},
    blocks::{
      basic_table_block::BasicTableBlock, owner_text_block::OwnerTextBlock,
      vendor_text_block::VendorTextBlock,
    },
    dimensions::owner_text_dimension::OwnerTextDimension,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl DBBlock {
  pub fn get_current_dimensions(&self) -> Result<HashSet<Uuid>> {
    let block_type = BlockTypes::from_str(&self.block_type)?;

    let bd = self
      .block_data
      .clone();

    let current_dims = match block_type {
      BlockTypes::BasicTable => {
        let mut block_data: BasicTableBlock = serde_json::from_value(bd)?;

        let mut dims: Vec<Uuid> = vec![];

        dims.append(&mut block_data.rows);
        dims.append(&mut block_data.columns);

        dims
      }
      BlockTypes::OwnerText => {
        let block_data: OwnerTextBlock = serde_json::from_value(bd)?;

        match block_data.content_dimension_id {
          Some(cdi) => vec![cdi],
          None => vec![],
        }
      }
      BlockTypes::VendorText => {
        let block_data: VendorTextBlock = serde_json::from_value(bd)?;

        match block_data.content_dimension_id {
          Some(cdi) => vec![cdi],
          None => vec![],
        }
      }
    };

    let current_dims_set: HashSet<Uuid> = current_dims
      .into_iter()
      .collect();

    Ok(current_dims_set)
  }

  pub fn remove_dimensions(&mut self, dimensions: Vec<Uuid>) -> Result<bool> {
    let block_type = BlockTypes::from_str(&self.block_type)
      .expect("Unable to convert db block block_type to BlockTypes enum");

    let bd = self
      .block_data
      .clone();

    let was_updated = match block_type {
      BlockTypes::BasicTable => {
        let mut block_data: BasicTableBlock = serde_json::from_value(bd)?;

        let dims_set: HashSet<Uuid> = dimensions
          .clone()
          .into_iter()
          .collect();
        let rows_set: HashSet<Uuid> = block_data
          .rows
          .clone()
          .into_iter()
          .collect();
        let columns_set: HashSet<Uuid> = block_data
          .columns
          .clone()
          .into_iter()
          .collect();

        let has_in_rows: Vec<&Uuid> = rows_set
          .intersection(&dims_set)
          .collect();
        let has_in_columns: Vec<&Uuid> = columns_set
          .intersection(&dims_set)
          .collect();

        if has_in_rows.len() > 0 || has_in_columns.len() > 0 {
          block_data
            .rows
            .retain(|r| !&dimensions.contains(&r));
          block_data
            .columns
            .retain(|r| !&dimensions.contains(&r));

          self.block_data = serde_json::to_value(block_data)?;

          true
        } else {
          false
        }
      }
      BlockTypes::OwnerText => {
        let mut block_data: OwnerTextBlock = serde_json::from_value(bd)?;

        if let Some(cdi) = block_data.content_dimension_id {
          if dimensions.contains(&cdi) {
            block_data.content_dimension_id = None;
            self.block_data = serde_json::to_value(block_data)?;
            true
          } else {
            false
          }
        } else {
          false
        }
      }
      BlockTypes::VendorText => {
        let mut block_data: VendorTextBlock = serde_json::from_value(bd)?;

        if let Some(cdi) = block_data.content_dimension_id {
          if dimensions.contains(&cdi) {
            block_data.content_dimension_id = None;
            self.block_data = serde_json::to_value(block_data)?;
            true
          } else {
            false
          }
        } else {
          false
        }
      }
    };

    Ok(was_updated)
  }
}

impl From<DBBlock> for DBUpdateBlock {
  fn from(db_block: DBBlock) -> Self {
    DBUpdateBlock {
      id: db_block.id,
      block_type: BlockTypes::from_str(&db_block.block_type)
        .expect("Unable to convert block_type string to BlockTypes Enum"),
      block_data: Some(db_block.block_data),
    }
  }
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

pub async fn get_portal_vendor_blocks<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  portal_id: Uuid,
) -> Result<Vec<DBBlock>> {
  sqlx::query_as!(
    DBBlock,
    "select * from blocks where portal_id = $1 and egress = 'vendor';",
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

// Does not perform any cleanup, just deletes a block in the db.
// You probably want to use clean_delete_block() below!
pub async fn dangerous_delete_block<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  block_id: Uuid,
) -> Result<i32> {
  sqlx::query!("delete from blocks where id = $1", block_id)
    .execute(pool)
    .await
    .map(|qr| qr.rows_affected() as i32)
    .map_err(anyhow::Error::from)
}

pub async fn clean_delete_block<'e>(
  pool: PgPool,
  auth0_user_id: &str,
  block_id: Uuid,
) -> Result<i32> {
  let mut tx = pool.begin().await?;

  // Below is a...naive aproach to disassociating dimensions that are used in an Owner block
  // from Vendor Blocks when the Owner block is deleted.
  // The logic goes that when an Owner Block is deleted, any Vendor Blocks that use dimensions (and cells)
  // that were originally created for the Owner Block should be deleted as well.
  // I think that at some point in the future we will need a Portal clean up (garbage collect) method to delete all
  // dims and cells Which are not connected to any blocks. It might also be good to add an
  // "owned_dimensions" column to the block table so that we can track what dimensions should be deleted

  let block = get_block(&mut tx, block_id).await?;

  if block
    .egress
    .contains("owner")
  {
    // Get all dimensions currently being used by the block that is to be destroyed
    let block_dims = block.get_current_dimensions()?;

    // Get all the vendor blocks in the portal
    let portal_vendor_blocks = get_portal_vendor_blocks(&mut tx, block.portal_id).await?;

    for mut vendor_block in portal_vendor_blocks {
      let vendor_block_dims = vendor_block.get_current_dimensions()?;

      let dims_in_both_blocks: Vec<Uuid> = block_dims
        .intersection(&vendor_block_dims)
        .map(|d| d.to_owned())
        .collect();

      if dims_in_both_blocks.len() > 0 {
        let updated = vendor_block.remove_dimensions(dims_in_both_blocks)?;

        if updated {
          update_block(&mut tx, auth0_user_id, vendor_block.into()).await?;
        }
      }
    }
  }

  let deleted_count = dangerous_delete_block(&mut tx, block_id).await?;

  tx.commit().await?;

  Ok(deleted_count)
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
    dimension_data: serde_json::to_value(OwnerTextDimension { empty: true })?,
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
