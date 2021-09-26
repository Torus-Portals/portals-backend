use juniper::{FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use serde_json;
use uuid::Uuid;

use crate::graphql::context::GQLContext;
use crate::graphql::schema::block::{BlockParts, BlockTypes, NewBlock};
use crate::graphql::schema::Mutation;
use crate::services::db::block_service::{create_owner_text_block};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct OwnerTextBlock {
  #[serde(rename = "contentDimensionId")]
  pub content_dimension_id: Option<Uuid>,
}

#[derive(GraphQLInputObject, Serialize, Deserialize)]
pub struct NewOwnerTextBlock {
  pub portal_id: Uuid,

  pub portal_view_id: Uuid,
}

impl From<NewOwnerTextBlock> for NewBlock {
  fn from(new_owner_text_block: NewOwnerTextBlock) -> Self {
    let block_data = OwnerTextBlock {
      content_dimension_id: None
    };

    NewBlock {
      block_type: BlockTypes::OwnerText,
      portal_id: new_owner_text_block.portal_id,
      portal_view_id: new_owner_text_block.portal_view_id,
      egress: String::from("owner"),
      block_data: serde_json::to_value(&block_data)
        .ok()
        .unwrap(), // replace this!!
    }
  }
}

impl Mutation {
  pub async fn create_owner_text_block_impl(
    ctx: &GQLContext,
    new_owner_text_block: NewOwnerTextBlock,
  ) -> FieldResult<BlockParts> {
    let local_pool = ctx.pool.clone();

    let portal_id = new_owner_text_block.portal_id.clone();
    let portal_view_id = new_owner_text_block.portal_view_id.clone();

    create_owner_text_block(
      local_pool,
      &ctx.auth0_user_id,
      portal_id,
      portal_view_id,
    )
    .await
    .map(|db_parts| db_parts.into())
    .map_err(FieldError::from)
  }
}
