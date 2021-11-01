use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::{graphql::schema::{connection_content::{ConnectionContent, ContentTypes}, sources::block_source::BlockSource}, services::db::source_service::DBSource};

use super::{
  block_service::{get_block, get_blocks},
  connection_service::get_connections,
  source_service::get_sources,
};

// block_destination_type: Table.

pub async fn get_connection_content(pool: PgPool, auth0id: &str, block_id: Uuid) -> Result<Vec<ConnectionContent>> {
  let mut tx = pool.begin().await?;

  // Get connections for a given block
  let connections = get_connections(&mut tx, block_id).await?;
  dbg!(&connections);

  // let destination_block = get_block(&mut tx, block_id).await?;

  let source_ids = connections
    .clone()
    .into_iter()
    .filter_map(|c| c.source_id)
    .collect::<Vec<Uuid>>();
  let sources = get_sources(&mut tx, source_ids).await?;
  dbg!(&sources);

  let source_block_pairs = sources
    .clone()
    .into_iter()
    .filter_map(|s| {
      match s
        .source_type
        .as_str()
      {
        "Block" => {
          let temp_s = s.clone();
          let source_data: BlockSource = serde_json::from_value(s.source_data).unwrap();
          Some((temp_s, source_data))
        }
        _ => None,
      }
    })
    .collect::<Vec<(DBSource, BlockSource)>>();
  dbg!(&source_block_pairs);

  let source_block_ids = source_block_pairs
    .iter()
    .map(|(_, bs)| bs.block_id.clone())
    .collect::<Vec<Uuid>>();

  let source_blocks = get_blocks(&mut tx, source_block_ids).await?;

  let conn_content = source_blocks.into_iter().map(|b| {
    let a = source_block_pairs
      .iter()
      .find(|(_, bs)| bs.block_id == b.id)
      .unwrap();

    ConnectionContent {
        source_id: a.0.id,
        content_type: ContentTypes::TableBlock,
        content_data: serde_json::to_string(&b.block_data).unwrap(),
    }
  }).collect::<Vec<ConnectionContent>>();

  dbg!(&conn_content);

  tx.commit().await?;

  Ok(conn_content)
}
