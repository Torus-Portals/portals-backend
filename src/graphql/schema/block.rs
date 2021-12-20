use chrono::{DateTime, Utc};
use juniper::{
  FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, GraphQLUnion,
};
use serde_json;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::blocks::cells_block::CellsBlock;
use super::blocks::empty_block::EmptyBlock;
use super::blocks::table_block::TableBlock;
use super::blocks::text_block::TextBlock;
use super::blocks::xy_chart_block::XYChartBlock;
use super::blocks::files_block::FilesBlock;

use super::block_configs::cells_block_config::CellsBlockConfig;
use super::block_configs::empty_block_config::EmptyBlockConfig;
use super::block_configs::table_block_config::TableBlockConfig;
use super::block_configs::text_block_config::TextBlockConfig;
use super::block_configs::xy_chart_block_config::XYChartBlockConfig;
use super::block_configs::files_block_config::FilesBlockConfig;

use super::Mutation;
use super::Query;
use crate::graphql::context::GQLContext;
use crate::services::db::block_service::DBBlock;
use crate::services::db::block_service::{
  create_block, delete_block, delete_blocks, get_block, get_blocks, get_page_blocks, update_block,
};

#[derive(Debug, Clone, GraphQLUnion, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub enum GQLBlocks {
  Empty(EmptyBlock),

  Table(TableBlock),

  Text(TextBlock),

  Cells(CellsBlock),

  XYChart(XYChartBlock),

  Files(FilesBlock),
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum BlockTypes {
  #[strum(serialize = "Table")]
  #[graphql(name = "Table")]
  Table,

  #[strum(serialize = "Text")]
  #[graphql(name = "Text")]
  Text,

  #[strum(serialize = "Cells")]
  #[graphql(name = "Cells")]
  Cells,

  #[strum(serialize = "XYChart")]
  #[graphql(name = "XYChart")]
  XYChart,

  #[strum(serialize = "Files")]
  #[graphql(name = "Files")]
  Files,
}

#[derive(Debug, Clone, GraphQLUnion, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub enum BlockConfigs {
  Empty(EmptyBlockConfig),
  Table(TableBlockConfig),
  Text(TextBlockConfig),
  Cells(CellsBlockConfig),
  XYChart(XYChartBlockConfig),
  Files(FilesBlockConfig),
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
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

  pub block_config: BlockConfigs,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl TryFrom<DBBlock> for Block {
  type Error = anyhow::Error;

  fn try_from(db_block: DBBlock) -> anyhow::Result<Self> {
    // TODO: Probably a better way to do this
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
      "Text" => {
        let b: TextBlock =
          serde_json::from_value(db_block.block_data).expect("unable to deserialize text block");
        GQLBlocks::Text(b)
      }
      "Cells" => {
        let b: CellsBlock =
          serde_json::from_value(db_block.block_data).expect("unable to deserialize cells block");
        GQLBlocks::Cells(b)
      }
      "XYChart" => {
        let b: XYChartBlock =
          serde_json::from_value(db_block.block_data).expect("unable to deserialize cells block");
        GQLBlocks::XYChart(b)
      }
      "Files" => {
        let b: FilesBlock =
          serde_json::from_value(db_block.block_data).expect("unable to deserialize files block");
        GQLBlocks::Files(b)
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

    let block_config = block_config_serde_value_to_block_config(&block_type, db_block.block_config)
      .expect("Unable to convert block_config string to block_config variant");

    Ok(Block {
      id: db_block.id,
      name: db_block.name,
      project_id: db_block.project_id,
      dashboard_id: db_block.dashboard_id,
      page_id: db_block.page_id,
      block_type,
      block_data,
      block_config,
      created_at: db_block.created_at,
      created_by: db_block.created_by,
      updated_at: db_block.updated_at,
      updated_by: db_block.updated_by,
    })
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
      .map(|db_block| db_block.try_into().map_err(FieldError::from))
      .map_err(FieldError::from)?
  }

  pub async fn blocks_impl(ctx: &GQLContext, block_ids: Vec<Uuid>) -> FieldResult<Vec<Block>> {
    get_blocks(&ctx.pool, block_ids)
      .await
      .map(|blocks| {
        blocks
          .into_iter()
          .filter_map(|b| b.try_into().ok())
          .collect()
      })
      .map_err(FieldError::from)
  }

  pub async fn page_blocks_impl(ctx: &GQLContext, page_id: Uuid) -> FieldResult<Vec<Block>> {
    get_page_blocks(&ctx.pool, page_id)
      .await
      .map(|db_blocks| {
        db_blocks
          .into_iter()
          .filter_map(|b| b.try_into().ok())
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
      .map(|db_block| db_block.try_into().map_err(FieldError::from))
      .map_err(FieldError::from)?
  }

  pub async fn update_block_impl(
    ctx: &GQLContext,
    updated_block: UpdateBlock,
  ) -> FieldResult<Block> {
    update_block(&ctx.pool, &ctx.auth0_user_id, updated_block.into())
      .await
      .map(|db_block| db_block.try_into().map_err(FieldError::from))
      .map_err(FieldError::from)?
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

pub fn block_config_serde_value_to_block_config(
  block_type: &BlockTypes,
  value: serde_json::Value,
) -> anyhow::Result<BlockConfigs> {
  let block_config = match block_type {
    BlockTypes::Table => {
      let c: TableBlockConfig = serde_json::from_value(value)?;
      BlockConfigs::Table(c)
    }
    BlockTypes::Text => {
      let c: TextBlockConfig = serde_json::from_value(value)?;
      BlockConfigs::Text(c)
    }
    BlockTypes::Cells => {
      let c: CellsBlockConfig = serde_json::from_value(value)?;
      BlockConfigs::Cells(c)
    }
    BlockTypes::XYChart => {
      let c: XYChartBlockConfig = serde_json::from_value(value)?;
      BlockConfigs::XYChart(c)
    }
    BlockTypes::Files => {
      let c: FilesBlockConfig = serde_json::from_value(value)?;
      BlockConfigs::Files(c)
    }
  };

  Ok(block_config)
}
