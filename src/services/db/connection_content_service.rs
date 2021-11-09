use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
  graphql::schema::{
    blocks::table_block::{
      TableBlock, TableBlockCell, TableBlockCellTypes, TableBlockColumnTypes, TableBlockCells, TableBlockRow
    },
    connection_content::{ConnectionContent, ContentTypes},
    sources::block_source::BlockSource,
  }
};

use super::{
  block_service::{get_blocks, DBBlock},
  connection_service::get_connections,
  source_service::{DBSource, get_sources},
  user_service::{get_user_by_auth0_id},
};

pub struct TableBlockQueryArgs {
  user_id: Uuid,
}

pub enum SourceQueryArgs {
  TableBlock(TableBlockQueryArgs),
}

pub fn contruct_sourcequery_args(user_id: Uuid) -> SourceQueryArgs {
  SourceQueryArgs::TableBlock(TableBlockQueryArgs { user_id })
}

// TODO: This whole function sucks, and needs to be refactored.
pub fn query_block_source(
  db_block: DBBlock,
  _db_source: DBSource,
  _block_source: BlockSource,
  query_args: SourceQueryArgs,
) -> Result<serde_json::Value> {

  let block_data = db_block.block_data.clone();
  let a = match db_block
    .block_type
    .as_str()
  {
    "Empty" => Ok(serde_json::Value::Null),
    "Table" => {
      let b: TableBlock =
        serde_json::from_value(db_block.block_data).expect("unable to deserialize table block");
      let user_id = match query_args {
        SourceQueryArgs::TableBlock(TableBlockQueryArgs { user_id }) => user_id,
      };

      // if one of the columns is of type "Member", then filter out rows that requesting user is not a member of.
      let has_member_column = b
        .columns
        .iter()
        .any(|c| c.column_type == TableBlockColumnTypes::Member);

      if has_member_column {
        let member_col = b
          .columns
          .clone()
          .into_iter()
          .find(|c| c.column_type == TableBlockColumnTypes::Member)
          .expect("member column not found");

        let filtered_rows = b
          .rows
          .clone()
          .into_iter()
          .filter(|r| {
            let member_cell = b
              .cells
              .clone()
              .into_iter()
              .find(|c| {
                c.cell_type == TableBlockCellTypes::Member
                  && c.row_id == r.id
                  && c.column_id == member_col.id
              });

            if let Some(m_cell) = member_cell {
              if m_cell.cell_type == TableBlockCellTypes::Member {
                if let TableBlockCells::TableBlockMemberCell(cell) = m_cell.cell_data {
                  cell
                    .member_ids
                    .contains(&user_id)
                } else {
                  false
                }
              } else {
                false
              }
            } else {
              false
            }
          })
          .collect::<Vec<TableBlockRow>>();

        let filtered_row_ids = filtered_rows
          .iter()
          .map(|r| r.id)
          .collect::<Vec<Uuid>>();

        let filtered_cells = b
          .cells
          .clone()
          .into_iter()
          .filter(|c| filtered_row_ids.contains(&c.row_id))
          .collect::<Vec<TableBlockCell>>();

        let mut new_b = b.clone();
        new_b.rows = filtered_rows;
        new_b.cells = filtered_cells;



        return Ok(serde_json::to_value(new_b)?);
      } else {
        Ok(block_data)
      }
    }
    &_ => {
      return Err(anyhow!("Block type not supported"));
    }
  };

  a
}

pub async fn get_connection_content(
  pool: PgPool,
  auth0id: &str,
  block_id: Uuid,
) -> Result<Vec<ConnectionContent>> {
  let mut tx = pool.begin().await?;

  let user = get_user_by_auth0_id(&mut tx, auth0id).await?;

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

  let conn_content = source_blocks
    .into_iter()
    .map(|b| {
      let a = source_block_pairs
        .iter()
        .find(|(_, bs)| bs.block_id == b.id)
        .unwrap();

      let db_block = b.clone();
      let db_source = a.0.clone();
      let block_source = a.1.clone();

      let query_args = contruct_sourcequery_args(user.id);
      let queried_content =
        query_block_source(db_block, db_source, block_source, query_args).unwrap();

      dbg!(&queried_content);

      ConnectionContent {
        source_id: a.0.id,
        content_type: ContentTypes::TableBlock,
        content_data: serde_json::to_string(&queried_content).unwrap(),
        // content_data: serde_json::to_string(&b.block_data).unwrap(),
      }
    })
    .collect::<Vec<ConnectionContent>>();

  dbg!(&conn_content);

  tx.commit().await?;

  Ok(conn_content)
}
