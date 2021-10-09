use chrono::{DateTime, Utc};
use juniper::{
  FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, GraphQLUnion,
};
use serde_json;
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::blocks::basic_table_block::BasicTableBlock;
use super::blocks::empty_block::EmptyBlock;
use super::blocks::integration_block::IntegrationBlock;
use super::blocks::owner_text_block::OwnerTextBlock;
use super::blocks::vendor_single_cell_block::VendorSingleCellBlock;
use super::blocks::vendor_text_block::VendorTextBlock;
use super::cell::Cell;
use super::dimension::Dimension;
use super::Mutation;
use super::Query;
use crate::graphql::context::GQLContext;
use crate::services::db::block_service::DBBlock;
use crate::services::db::block_service::DBBlockParts;
use crate::services::db::block_service::{
  clean_delete_block, delete_blocks, get_block, get_blocks, create_block, update_block,
};

#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub enum GQLBlocks {
  Empty(EmptyBlock),
  Integration(IntegrationBlock),

  // Owner Blocks
  OwnerText(OwnerTextBlock),
  BasicTable(BasicTableBlock),

  // Vendor Blocks
  VendorText(VendorTextBlock),
  VendorSingleCell(VendorSingleCellBlock),
}

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum BlockTypes {
  #[strum(serialize = "Integration")]
  Integration,

  #[strum(serialize = "BasicTable")]
  #[graphql(name = "BasicTable")]
  BasicTable,

  #[strum(serialize = "OwnerText")]
  #[graphql(name = "OwnerText")]
  OwnerText,

  #[strum(serialize = "VendorText")]
  #[graphql(name = "VendorText")]
  VendorText,

  #[strum(serialize = "VendorSingleCell")]
  #[graphql(name = "VendorSingleCell")]
  VendorSingleCell,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub struct Block {
  pub id: Uuid,

  #[serde(rename = "blockType")]
  pub block_type: BlockTypes,

  pub portal_id: Uuid,

  #[serde(rename = "portalViewId")]
  pub portal_view_id: Uuid,

  pub egress: String,

  #[serde(rename = "blockData")]
  pub block_data: GQLBlocks,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

impl From<DBBlock> for Block {
  fn from(db_block: DBBlock) -> Self {
    // let a = serde_json::to_string(&db_block).expect("blah");
    // println!("{}", a);

    // let q = db_block.block_type;
    // println!("qqq {}", q);

    let block_data = match db_block
      .block_type
      .as_str()
    {
      "BasicTable" => {
        let b: BasicTableBlock = serde_json::from_value(db_block.block_data).expect("come on");
        GQLBlocks::BasicTable(b)
      }
      "OwnerText" => {
        let t: OwnerTextBlock =
          serde_json::from_value(db_block.block_data).expect("not OwnerText??");
        GQLBlocks::OwnerText(t)
      }
      "VendorText" => {
        let t: VendorTextBlock =
          serde_json::from_value(db_block.block_data).expect("not VendorText??");
        GQLBlocks::VendorText(t)
      }
      "Integration" => {
        let b: IntegrationBlock = serde_json::from_value(db_block.block_data)
          .expect("Unable to deserialize DBBlock into IntegrationBlock.");
        GQLBlocks::Integration(b)
      }
      &_ => GQLBlocks::Empty(EmptyBlock {
        block_type: String::from("nothing"),
      }),
    };

    let block_type = BlockTypes::from_str(
      db_block
        .block_type
        .as_str(),
    )
    .expect("Unable to convert block_type string to enum variant");

    Block {
      id: db_block.id,
      block_type,
      portal_id: db_block.portal_id,
      portal_view_id: db_block.portal_view_id,
      egress: db_block.egress,
      block_data,
      created_at: db_block.created_at,
      created_by: db_block.created_by,
      updated_at: db_block.updated_at,
      updated_by: db_block.updated_by,
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewBlock {
  pub block_type: BlockTypes,

  pub portal_id: Uuid,

  pub portal_view_id: Uuid,

  pub egress: String,

  pub block_data: String,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct UpdateBlock {
  pub id: Uuid,

  pub block_type: BlockTypes,

  #[graphql(description = "For now block_data needs to be stringified")]
  pub block_data: Option<String>,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub struct BlockParts {
  blocks: Vec<Block>,
  dimensions: Vec<Dimension>,
  cells: Vec<Cell>,
}

impl From<DBBlockParts> for BlockParts {
  fn from(db_block_parts: DBBlockParts) -> Self {
    BlockParts {
      blocks: db_block_parts
        .blocks
        .into_iter()
        .map(|b| b.into())
        .collect(),
      dimensions: db_block_parts
        .dimensions
        .into_iter()
        .map(|d| d.into())
        .collect(),
      cells: db_block_parts
        .cells
        .into_iter()
        .map(|c| c.into())
        .collect(),
    }
  }
}

impl Query {
  pub async fn block_impl(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Block> {
    get_block(&ctx.pool, block_id)
      .await
      .map(|db_block| db_block.into())
      .map_err(FieldError::from)
  }

  pub async fn blocks_impl(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Vec<Block>> {
    get_blocks(&ctx.pool, portal_id)
      .await
      .map(|db_blocks| {
        db_blocks
          .into_iter()
          .map(|b| b.into())
          .collect()
      })
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_block(ctx: &GQLContext, new_block: NewBlock) -> FieldResult<Block> {
    create_block(&ctx.pool, &ctx.auth0_user_id, new_block.into())
      .await
      .map(|db_block| db_block.into())
      .map_err(FieldError::from)
  }

  pub async fn update_block_impl(
    ctx: &GQLContext,
    updated_block: UpdateBlock,
  ) -> FieldResult<Block> {
    update_block(&ctx.pool, &ctx.auth0_user_id, updated_block.into())
      .await
      .map(|db_block| db_block.into())
      .map_err(FieldError::from)
  }

  pub async fn delete_block(ctx: &GQLContext, block_id: Uuid) -> FieldResult<i32> {
    let local_pool = ctx.pool.clone();

    clean_delete_block(local_pool, &ctx.auth0_user_id, block_id)
      .await
      .map_err(FieldError::from)
  }

  pub async fn delete_blocks(ctx: &GQLContext, block_ids: Vec<Uuid>) -> FieldResult<i32> {
    delete_blocks(&ctx.pool, block_ids)
      .await
      .map_err(FieldError::from)
  }
}
