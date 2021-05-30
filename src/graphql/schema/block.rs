use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLObject, GraphQLUnion, GraphQLEnum};
use serde_json;
use uuid::Uuid;
use std::str::FromStr;
use strum_macros::EnumString;

// use super::Mutation;
use super::Query;
use crate::graphql::context::GQLContext;
use crate::services::db::block_service::{DBBlock};

// #[derive(From, Debug, GraphQLUnion)]
#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
pub enum GQLBlocks {
  BasicTable(BasicTableBlock),
  Empty(EmptyBlock)
}

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString)]
pub enum BlockTypes {
  BasicTable
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct Block {
  pub id: Uuid,

  #[serde(rename = "blockType")]
  pub block_type: BlockTypes,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  #[serde(rename = "portalViewId")]
  pub portal_view_id: Uuid,

  pub egress: String,

  pub bbox: Vec<i32>,

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

    let block_data = match db_block.block_type.as_str() {
      "BasicTable" => {
        let b: BasicTableBlock = serde_json::from_value(db_block.data).expect("come on");
        GQLBlocks::BasicTable(b)
      },
      &_ => GQLBlocks::Empty(EmptyBlock { block_type: String::from("nothing")}),
    };


    let block_type = BlockTypes::from_str(db_block.block_type.as_str()).unwrap();

    Block {
      id: db_block.id,
      block_type,
      portal_id: db_block.portal_id,
      portal_view_id: db_block.portal_view_id,
      egress: db_block.egress,
      bbox: db_block.bbox,
      block_data,
      created_at: db_block.created_at,
      created_by: db_block.created_by,
      updated_at: db_block.updated_at,
      updated_by: db_block.updated_by,
    }
  }
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct BasicTableBlock {
  pub rows: Vec<Uuid>,

  pub columns: Vec<Uuid>,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct EmptyBlock { block_type: String }

// impl BasicTableBlock {
//   fn new() -> BasicTableBlock {
//     BasicTableBlock {
//       id: Uuid::new_v4(),
//       block_type: String::from("BasicTable"),
//       rows: vec![],
//       columns: vec![],
//     }
//   }
// }

impl Query {
  pub async fn block_impl(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Block> {
    ctx
      .db
      .get_block(block_id)
      .await
      .map(|db_block| db_block.into())
      .map_err(FieldError::from)
  }
}
