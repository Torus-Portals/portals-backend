use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
  graphql::schema::{
    block::{Block, BlockTypes, GQLBlocks},
    blocks::{table_block::TableBlock, xy_chart_block::XYChartBlock},
    connection_content::{ConnectionContent, ConnectionContentData, ContentTypes, SourceQueryArgs},
    sourcequeries::table_block_sourcequery::{query_table_block, TableBlockSourceQuery},
  },
  utils::ir::{IRSink, IRTap, Node},
};

use super::{
  block_service::DBBlock,
  connection_service::{get_connection_data, get_connections, DBConnection},
  source_service::DBSource,
  sourcequery_service::DBSourceQuery,
};

#[derive(Debug)]
pub struct ConnectionData {
  pub connection: DBConnection,
  pub source: DBSource,
  pub sourcequery: Option<DBSourceQuery>,
}

#[derive(Debug)]
pub struct BlockConnectionData {
  pub connection: DBConnection,
  pub source: Option<DBSource>,
  pub sourcequery: Option<DBSourceQuery>,
  pub blocks: Option<DBBlock>,
  // TODO: might want to add destination stuff to this struct.
}
/*
pub fn query_source_block(source_block, sourcequery, sourcequery_args) -> Block {

}
*/

// pub fn parse_sourcequery_args(
//   sourcequery_args_string: String,
//   block_type: &BlockTypes,
// ) -> Result<SourceQueryArgs> {
//   let sourcequery_args = match block_type {
//     BlockTypes::Table => {
//       let args = serde_json::from_str(&sourcequery_args_string)?;
//       SourceQueryArgs::TableBlockSourceQueryArgs(args)
//     }
//     BlockTypes::Text => {
//       let args = serde_json::from_str(&sourcequery_args_string)?;
//       SourceQueryArgs::TextBlockSourceQueryArgs(args)
//     }
//     BlockTypes::Cells => {
//       let args = serde_json::from_str(&sourcequery_args_string)?;
//       SourceQueryArgs::CellsBlockSourceQueryArgs(args)
//     }
//     BlockTypes::XYChart => {
//       let args = serde_json::from_str(&sourcequery_args_string)?;
//       SourceQueryArgs::XYChartBlockSourceQueryArgs(args)
//     }
//     BlockTypes::Files => {
//       let args = serde_json::from_str(&sourcequery_args_string)?;
//       SourceQueryArgs::FilesBlockSourceQueryArgs(args)
//     }
//   };

//   Ok(sourcequery_args)
// }

pub fn query_block(
  source_block: Block,
  sourcequery: Option<DBSourceQuery>,
  sourcequery_args: SourceQueryArgs,
) -> Result<Block> {
  // let block_type = source_block.block_type.clone();
  // let sourcequery_args = parse_sourcequery_args(sourcequery_args_string, &block_type)?;
  let mut block = source_block.clone();

  // actually query the block.
  let queried_block = match source_block.block_data {
    GQLBlocks::Table(table_block) => {
      if let Some(sq) = sourcequery {
        // let args: TableBlockSourceQueryArgs = serde_json::from_str(&sourcequery_args_string)?;
        let ssq: TableBlockSourceQuery = serde_json::from_value(sq.sourcequery_data)?;

        let queries_table_block = query_table_block(table_block, ssq, sourcequery_args)?;

        block.block_data = GQLBlocks::Table(queries_table_block);
      };

      block
    }
    GQLBlocks::Text(_) => todo!(),
    GQLBlocks::Cells(_) => todo!(),
    GQLBlocks::XYChart(_) => todo!(),
    GQLBlocks::Files(_) => todo!(),
    GQLBlocks::Empty(_) => todo!(),
  };

  Ok(queried_block)
}

pub async fn get_connection_content(
  pool: PgPool,
  auth0id: &str,
  block_id: Uuid,
  sourcequery_args: SourceQueryArgs,
) -> Result<Vec<ConnectionContent>> {
  let mut tx = pool.begin().await?;

  // let user = get_user_by_auth0_id(&mut tx, auth0id).await?;
  // TODO: Need to check if user has access to this block.

  // Get connections for a given block
  let connections = get_connections(&mut tx, block_id).await?;

  let mut connection_content = Vec::new();

  for conn in connections.into_iter() {
    let conn_pool = pool.clone();

    let connection_data = get_connection_data(conn_pool, conn.id).await?;

    if let Some(source_block) = connection_data.source_block {
      let queried_block = query_block(
        source_block.clone(),
        connection_data.sourcequery,
        sourcequery_args.clone(),
      )?;

      let tapped = match queried_block.block_data {
        GQLBlocks::Table(block_table) => {
          block_table.tap()
        },
        _ => {
          Node {
            id: Uuid::new_v4(),
            label: "".to_string(),
            debug_info: "".to_string(),
            data: None,
          }
        }
          // GQLBlocks::Empty(_) => todo!(),
          // GQLBlocks::Text(_) => todo!(),
          // GQLBlocks::Cells(_) => todo!(),
          // GQLBlocks::XYChart(_) => todo!(),
          // GQLBlocks::Files(_) => todo!(),
      };

      if let Some(destination_block) = connection_data.destination_block {
        let content_data = match destination_block.block_type {
          BlockTypes::Table => ConnectionContentData::TableBlock(TableBlock::sink(tapped)),
          BlockTypes::Text => todo!(),
          BlockTypes::Cells => todo!(),
          BlockTypes::XYChart => ConnectionContentData::XYChartBlock(XYChartBlock::sink(tapped)),
          BlockTypes::Files => todo!(),
        };

        let content_type = match destination_block.block_type {
          BlockTypes::Table => ContentTypes::TableBlock,
          BlockTypes::Text => ContentTypes::TextBlock,
          BlockTypes::Cells => ContentTypes::CellsBlock,
          BlockTypes::XYChart => ContentTypes::XYChartBlock,
          BlockTypes::Files => ContentTypes::FilesBlock,
        };

        connection_content.push(ConnectionContent {
          source_id: source_block
            .id
            .clone(),
          connection_id: conn.id.clone(),
          content_type,
          content_data,
        });
      }
    }
  }

  Ok(connection_content)
}
