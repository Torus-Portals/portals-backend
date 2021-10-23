use juniper::{graphql_value, FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use serde_json;
use uuid::Uuid;

use crate::graphql::context::GQLContext;
use crate::graphql::schema::block::{Block, BlockParts, BlockTypes, GQLBlocks, NewBlock};
use crate::graphql::schema::integration::{Integration};
use crate::graphql::schema::{Mutation, Query};
use crate::services::db::block_service::{create_integration_block, get_block};
use crate::services::db::integration_service::get_integration;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub struct IntegrationBlock {
  // TODO: block-specific or portal-specific?
  pub integration_id: Option<Uuid>,

  pub row_dim: Option<Uuid>,

  pub col_dim: Option<Uuid>,
}

#[derive(GraphQLInputObject, Serialize, Deserialize)]
pub struct NewIntegrationBlock {
  pub portal_id: Uuid,

  pub portal_view_id: Uuid,
  // TODO: may want to use an existing Integration instead of creating NewIntegration
  // Especially if integrations can be "managed" separately
  //   pub integration: NewIntegration,
  pub integration_id: Uuid,

  pub row_dim: String,

  pub col_dim: String,
}

impl From<NewIntegrationBlock> for NewBlock {
  fn from(new_integration_block: NewIntegrationBlock) -> Self {
    let block_data = IntegrationBlock {
      integration_id: Some(new_integration_block.integration_id),
      row_dim: None,
      col_dim: None,
    };

    NewBlock {
      block_type: BlockTypes::Integration,
      portal_id: new_integration_block.portal_id,
      portal_view_id: new_integration_block.portal_view_id,
      egress: "Integration".to_string(),
      block_data: serde_json::to_string(&block_data)
      .expect("Unable to serialize IntegrationBlock into valid JSON format."),
    }
  }
}

impl Query {
  // Returns the Integration associated with an IntegrationBlock.
  pub async fn integration_block_options_impl(
    ctx: &GQLContext,
    block_id: Uuid,
  ) -> FieldResult<Integration> {
    let block: Block = get_block(&ctx.pool, block_id)
      .await
      .map(|db_block| db_block.into())
      .map_err(FieldError::from)?;

    match &block.block_data {
      GQLBlocks::Integration(block) if block.integration_id.is_some() => {
        get_integration(&ctx.pool, block.integration_id.unwrap())
          .await
          .map(|db_integration| db_integration.into())
          .map_err(FieldError::from)
      }
      GQLBlocks::Integration(block) => Err(FieldError::new(
        "No Integration associated with this IntegrationBlock",
        graphql_value!(format!("Block ID: {}", block_id).as_str()),
      )),
      _ => Err(FieldError::new(
        "Block is not of IntegrationBlock type",
        graphql_value!(format!("Block ID: {}", block_id).as_str()),
      )),
    }
  }
}

impl Mutation {
  pub async fn create_integration_block_impl(
    ctx: &GQLContext,
    new_integration_block: NewIntegrationBlock,
  ) -> FieldResult<BlockParts> {
    create_integration_block(ctx.pool.clone(), &ctx.auth0_user_id, new_integration_block)
      .await
      .map(|db_parts| db_parts.into())
      .map_err(FieldError::from)
  }
}
