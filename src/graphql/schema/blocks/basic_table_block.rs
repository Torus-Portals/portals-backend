use juniper::{FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use serde_json;
use uuid::Uuid;

use crate::graphql::context::GQLContext;
use crate::graphql::schema::block::{Block, BlockTypes, NewBlock};
use crate::graphql::schema::Mutation;
use crate::services::db::block_service::create_block;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct BasicTableBlock {
  pub rows: Vec<Uuid>,

  pub columns: Vec<Uuid>,
}

#[derive(GraphQLInputObject, Serialize, Deserialize)]
pub struct NewBasicTableBlock {
  pub portal_id: Uuid,

  pub portal_view_id: Uuid,

  pub egress: String,

  pub rows: Vec<Uuid>,

  pub columns: Vec<Uuid>,
}

impl From<NewBasicTableBlock> for NewBlock {
  fn from(new_basic_table_block: NewBasicTableBlock) -> Self {
    let block_data = BasicTableBlock {
      rows: new_basic_table_block.rows,
      columns: new_basic_table_block.columns,
    };

    NewBlock {
      block_type: BlockTypes::BasicTable,
      portal_id: new_basic_table_block.portal_id,
      portal_view_id: new_basic_table_block.portal_view_id,
      egress: new_basic_table_block.egress,
      block_data: serde_json::to_value(&block_data)
        .ok()
        .unwrap(), // replace this!!
    }
  }
}

impl Mutation {
  pub async fn create_basic_table_impl(
    ctx: &GQLContext,
    new_basic_table_block: NewBasicTableBlock,
  ) -> FieldResult<Block> {
    let new_block: NewBlock = new_basic_table_block.into();

      create_block(&ctx.pool, &ctx.auth0_user_id, new_block.into())
      .await
      .map(|db_block| db_block.into())
      .map_err(FieldError::from)
  }
}
