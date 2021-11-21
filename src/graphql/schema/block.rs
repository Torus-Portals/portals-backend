use chrono::{DateTime, Utc};
use juniper::{
  FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, GraphQLUnion,
};
use serde_json;
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::blocks::empty_block::EmptyBlock;
use super::blocks::table_block::TableBlock;

use super::Mutation;
use super::Query;
use crate::graphql::context::GQLContext;
use crate::services::db::block_service::DBBlock;
use crate::services::db::block_service::{
  create_block, delete_block, delete_blocks, get_block, get_blocks, get_page_blocks, update_block,
};

#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub enum GQLBlocks {
  Empty(EmptyBlock),

  Table(TableBlock),
}

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum BlockTypes {
  #[strum(serialize = "Table")]
  #[graphql(name = "Table")]
  Table,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
#[serde(rename_all = "camelCase")]
pub struct Block {
  pub id: Uuid,

  pub name: String,

  pub project_id: Uuid,

  pub dashboard_id: Uuid,

  pub page_id: Uuid,

  pub block_type: BlockTypes,

  pub block_data: GQLBlocks,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl From<DBBlock> for Block {
  fn from(db_block: DBBlock) -> Self {
    let block_data = match db_block
      .block_type
      .as_str()
    {
      "Empty" => {
        let b: EmptyBlock =
          serde_json::from_value(db_block.block_data).expect("unable to deserialize empty block");
        GQLBlocks::Empty(b)
      }
      "Table" => {
        let b: TableBlock =
          serde_json::from_value(db_block.block_data).expect("unable to deserialize table block");
        GQLBlocks::Table(b)
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
      name: db_block.name,
      project_id: db_block.project_id,
      dashboard_id: db_block.dashboard_id,
      page_id: db_block.page_id,
      block_type,
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
  pub project_id: Uuid,

  pub dashboard_id: Uuid,

  pub page_id: Uuid,

  pub block_type: BlockTypes,

  pub block_data: String,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct UpdateBlock {
  pub id: Uuid,

  pub name: Option<String>,

  pub block_type: BlockTypes,

  #[graphql(description = "For now block_data needs to be stringified")]
  pub block_data: Option<String>,
}

impl Query {
  pub async fn block_impl(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Block> {
    get_block(&ctx.pool, block_id)
      .await
      .map(|db_block| db_block.into())
      .map_err(FieldError::from)
  }

  pub async fn blocks_impl(ctx:&GQLContext, block_ids: Vec<Uuid>) -> FieldResult<Vec<Block>> {
    get_blocks(&ctx.pool, block_ids)
      .await
      .map(|blocks| blocks.into_iter().map(|b| b.into()).collect())
      .map_err(FieldError::from)
  }

  pub async fn page_blocks_impl(ctx: &GQLContext, page_id: Uuid) -> FieldResult<Vec<Block>> {
    get_page_blocks(&ctx.pool, page_id)
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
    let local_pool = ctx.pool.clone();

    create_block(local_pool, &ctx.auth0_user_id, new_block.into())
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

  pub async fn delete_block(ctx: &GQLContext, block_id: Uuid) -> FieldResult<DateTime<Utc>> {
    delete_block(&ctx.pool, &ctx.auth0_user_id, block_id)
      .await
      .map_err(FieldError::from)
  }

  pub async fn delete_blocks(ctx: &GQLContext, block_ids: Vec<Uuid>) -> FieldResult<i32> {
    delete_blocks(&ctx.pool, block_ids)
      .await
      .map_err(FieldError::from)
  }
}
