use juniper::{FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use serde_json;
use uuid::Uuid;

use crate::graphql::context::GQLContext;
use crate::graphql::schema::block::{Block, BlockTypes, NewBlock};
use crate::graphql::schema::Mutation;
use crate::services::db::block_service::{create_vendor_text_block};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct VendorTextBlock {
  #[serde(rename = "contentDimensionId")]
  pub content_dimension_id: Option<Uuid>,
}

#[derive(GraphQLInputObject, Serialize, Deserialize)]
pub struct NewVendorTextBlock {
  pub portal_id: Uuid,

  pub portal_view_id: Uuid,
}

impl From<NewVendorTextBlock> for NewBlock {
  fn from(new_vendor_text_block: NewVendorTextBlock) -> Self {
    let block_data = VendorTextBlock {
      content_dimension_id: None
    };

    NewBlock {
      block_type: BlockTypes::VendorText,
      portal_id: new_vendor_text_block.portal_id,
      portal_view_id: new_vendor_text_block.portal_view_id,
      egress: String::from("vendor"),
      block_data: serde_json::to_value(&block_data)
        .ok()
        .unwrap(), // replace this!!
    }
  }
}

impl Mutation {
  pub async fn create_vendor_text_block_impl(
    ctx: &GQLContext,
    new_vendor_text_block: NewVendorTextBlock,
  ) -> FieldResult<Block> {
    let local_pool = ctx.pool.clone();

    let portal_id = new_vendor_text_block.portal_id.clone();
    let portal_view_id = new_vendor_text_block.portal_view_id.clone();

    create_vendor_text_block(
      local_pool,
      &ctx.auth0_user_id,
      portal_id,
      portal_view_id,
    )
    .await
    .map(|db_block| db_block.into())
    .map_err(FieldError::from)
  }
}
