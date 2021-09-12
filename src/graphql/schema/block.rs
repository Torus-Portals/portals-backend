use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLEnum, GraphQLObject, GraphQLUnion};
use serde_json;
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::blocks::basic_table_block::BasicTableBlock;
use super::blocks::empty_block::EmptyBlock;
use super::Mutation;
use super::Query;
use crate::graphql::context::GQLContext;
use crate::services::db::block_service::{get_block, get_blocks, delete_block, delete_blocks};
use crate::services::db::block_service::DBBlock;

#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
pub enum GQLBlocks {
  BasicTable(BasicTableBlock),
  Empty(EmptyBlock),
}

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum BlockTypes {
  #[strum(serialize = "BasicTable")]
  BasicTable,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
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

pub struct NewBlock {
  pub block_type: BlockTypes,

  pub portal_id: Uuid,

  pub portal_view_id: Uuid,

  pub egress: String,

  pub block_data: serde_json::Value, // For now the json should be stringified
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
  // Not using at the moment, due to no good way currently to type a json field.
  // Will create separate mutations for each block type
  //   pub async fn create_block(ctx: &GQLContext, new_block: NewBlock) -> FieldResult<Block> {
  //     ctx
  //       .db
  //       .create_block(&ctx.auth0_user_id, new_block.into())
  //       .await
  //       .map(|b| b.into())
  //       .map_err(FieldError::from)
  //   }

  pub async fn delete_block(ctx: &GQLContext, block_id: Uuid) -> FieldResult<i32> {
      delete_block(&ctx.pool, block_id)
      .await
      .map_err(FieldError::from)
  }

  pub async fn delete_blocks(ctx: &GQLContext, block_ids: Vec<Uuid>) -> FieldResult<i32> {
      delete_blocks(&ctx.pool, block_ids)
      .await
      .map_err(FieldError::from)
  }
}
