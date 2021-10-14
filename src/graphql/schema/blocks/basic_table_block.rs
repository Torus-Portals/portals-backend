use juniper::{FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use serde_json;
use uuid::Uuid;

use crate::graphql::context::GQLContext;
use crate::graphql::schema::block::{Block, BlockTypes, NewBlock};
use crate::graphql::schema::Mutation;
use crate::services::db::block_service::create_block;

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicTableRow {
  pub portal_member_dimension: Option<Uuid>,

  pub row_index: i32,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicTableColumn {
  pub dimension: Uuid,
}

#[derive(GraphQLInputObject, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewBasicTableRow {
  pub portal_member_dimension: Option<Uuid>,

  pub row_index: i32,
}

#[derive(GraphQLInputObject, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewBasicTableColumn {
  pub dimension: Uuid,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct BasicTableBlock {
  pub rows: Vec<BasicTableRow>,

  pub columns: Vec<BasicTableColumn>,
}

#[derive(GraphQLInputObject, Serialize, Deserialize)]
pub struct NewBasicTableBlock {
  pub portal_id: Uuid,

  pub portal_view_id: Uuid,

  pub egress: String,

  pub rows: Vec<NewBasicTableRow>,

  pub columns: Vec<NewBasicTableColumn>,
}

impl From<NewBasicTableBlock> for NewBlock {
  fn from(new_basic_table_block: NewBasicTableBlock) -> Self {
    let block_data = BasicTableBlock {
      rows: new_basic_table_block
        .rows
        .into_iter()
        .map(|r| BasicTableRow {
          portal_member_dimension: r.portal_member_dimension,
          row_index: r.row_index,
        })
        .collect(),
      columns: new_basic_table_block
        .columns
        .into_iter()
        .map(|c| BasicTableColumn {
          dimension: c.dimension,
        })
        .collect(),
    };

    NewBlock {
      block_type: BlockTypes::BasicTable,
      portal_id: new_basic_table_block.portal_id,
      portal_view_id: new_basic_table_block.portal_view_id,
      egress: new_basic_table_block.egress,
      block_data: serde_json::to_string(&block_data).ok().unwrap(),
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
