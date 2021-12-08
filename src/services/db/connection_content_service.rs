use anyhow::{Result};
use sqlx::{PgPool};
use uuid::Uuid;

use crate::{
  graphql::schema::{
    block::{BlockTypes, GQLBlocks},
    blocks::{table_block::TableBlock, xy_chart_block::XYChartBlock},
    connection_content::{ConnectionContent, ConnectionContentData, ContentTypes},
  },
  utils::ir::{IRSink, IRTap, Node},
};

use super::{
  block_service::{DBBlock},
  connection_service::{get_connection_data, get_connections, DBConnection},
  source_service::{DBSource},
  sourcequery_service::{DBSourceQuery},
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

pub async fn get_connection_content(
  pool: PgPool,
  auth0id: &str,
  block_id: Uuid,
) -> Result<Vec<ConnectionContent>> {
  let mut tx = pool.begin().await?;

  // let user = get_user_by_auth0_id(&mut tx, auth0id).await?;

  // Get connections for a given block
  let connections = get_connections(&mut tx, block_id).await?;

  let mut connection_content = Vec::new();

  for conn in connections.into_iter() {
    let conn_pool = pool.clone();

    let connection_data = get_connection_data(conn_pool, conn.id).await?;

    if let Some(source_block) = connection_data.source_block {
      let tapped = match source_block.block_data {
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
      };

      if let Some(destination_block) = connection_data.destination_block {
        let content_data = match destination_block.block_type {
          BlockTypes::Table => ConnectionContentData::TableBlock(TableBlock::sink(tapped)),
          BlockTypes::Text => todo!(),
          BlockTypes::Cells => todo!(),
          BlockTypes::XYChart => ConnectionContentData::XYChartBlock(XYChartBlock::sink(tapped)),
        };

        let content_type = match destination_block.block_type {
          BlockTypes::Table => ContentTypes::TableBlock,
          BlockTypes::Text => ContentTypes::TextBlock,
          BlockTypes::Cells => ContentTypes::CellsBlock,
          BlockTypes::XYChart => ContentTypes::XYChartBlock,
        };

        connection_content.push(ConnectionContent {
          source_id: source_block
            .id
            .clone(),
          content_type,
          content_data,
        });
      }
    }
  }

  Ok(connection_content)
}
