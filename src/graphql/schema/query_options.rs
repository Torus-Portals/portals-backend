use juniper::{
  FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, GraphQLUnion,
};
use std::convert::{TryFrom, TryInto};

use uuid::Uuid;

use super::{
  Query,
  block::{Block, BlockTypes, GQLBlocks}, blocks::table_block::TableBlockColumn,
};
use crate::{graphql::context::GQLContext, services::db::block_service::get_block};


#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct TableQueryOptions {
  pub member: Option<bool>, // Can member be filtered?
  pub columns: Option<Vec<TableBlockColumn>>,
}

#[derive(Debug, Clone, GraphQLUnion, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub enum BQOptions {
  Table(TableQueryOptions),
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
#[serde(rename_all = "camelCase")]
pub struct BlockQueryOptions {
  pub block_id: Uuid,

  pub block_type: BlockTypes,

  pub options: BQOptions,
}

impl Query {
  pub async fn block_query_options_impl(
    ctx: &GQLContext,
    block_id: Uuid,
  ) -> FieldResult<BlockQueryOptions> {
    let db_block = get_block(&ctx.pool, block_id).await?;
    let block: Block = db_block.try_into()?;

    let o = match block.block_data {
        GQLBlocks::Empty(_) => todo!(),
        GQLBlocks::Table(table_block) => {
          let oo = TableQueryOptions {
            member: Some(true),
            columns: Some(table_block.columns.clone()),
          };

          BQOptions::Table(oo)
        },
        GQLBlocks::Text(_) => todo!(),
        GQLBlocks::Cells(_) => todo!(),
        GQLBlocks::XYChart(_) => todo!(),
        GQLBlocks::Files(_) => todo!(),
    };


    Ok(BlockQueryOptions {
      block_id,
      block_type: block.block_type,
      options: o,
    })
  }
}
