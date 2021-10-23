use juniper::{GraphQLInputObject, GraphQLObject};
use serde_json;
use uuid::Uuid;

use crate::graphql::schema::block::{BlockTypes, NewBlock};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorSingleCellBlock {
  pub dimension: Option<Uuid>,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewVendorSingleCellBlock {
  pub portal_id: Uuid,

  pub portal_view_id: Uuid,

  dimension: Option<Uuid>,
}

impl From<NewVendorSingleCellBlock> for NewBlock {
  fn from(new_vendor_single_cell_block: NewVendorSingleCellBlock) -> Self {
    let block_data = VendorSingleCellBlock {
      dimension: new_vendor_single_cell_block.dimension
    };

    NewBlock {
      block_type: BlockTypes::VendorSingleCell,
      portal_id: new_vendor_single_cell_block.portal_view_id,
      portal_view_id: new_vendor_single_cell_block.portal_view_id,
      egress: String::from("vendor"),
      block_data: serde_json::to_string(&block_data).ok().unwrap(),
    }
  }
}
