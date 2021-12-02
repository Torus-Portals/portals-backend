use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;
use std::convert::TryInto;

use crate::{
  graphql::schema::{
    block::{Block, GQLBlocks},
    connection_content::{ConnectionContent, ContentTypes},
    sourcequeries::table_block_sourcequery::{query_table_block, TableBlockQueryArgs},
    sourcequery::{GQLSourceQueries, SourceQuery},
    sources::block_source::BlockSource,
  },
  utils::ir::{IRSink, IRTap},
};

use super::{
  block_service::{get_blocks, DBBlock},
  connection_service::{get_connections, DBConnection},
  source_service::{get_sources, DBSource},
  sourcequery_service::{get_sourcequeries, DBSourceQuery},
  user_service::get_user_by_auth0_id,
};

// pub enum SourceQueryArgs {
//   TableBlock(TableBlockQueryArgs),
// }

// pub fn construct_sourcequery_args(user_id: Uuid) -> SourceQueryArgs {
//   SourceQueryArgs::TableBlock(TableBlockQueryArgs { user_id })
// }

// TODO: This whole function sucks, and needs to be refactored.
// pub fn query_block_source(
//   db_block: DBBlock,
//   _db_source: DBSource,
//   _block_source: BlockSource,
//   query_args: SourceQueryArgs,
// ) -> Result<serde_json::Value> {

//   let block_data = db_block.block_data.clone();
//   let a = match db_block
//     .block_type
//     .as_str()
//   {
//     "Empty" => Ok(serde_json::Value::Null),
//     "Table" => {
//       let b: TableBlock =
//         serde_json::from_value(db_block.block_data).expect("unable to deserialize table block");
//       let user_id = match query_args {
//         SourceQueryArgs::TableBlock(TableBlockQueryArgs { user_id }) => user_id,
//       };

//       // Just testing tap out.
//       let tapped = b.tap();
//       // dbg!(&tapped);

//       let _aaa = b.sink(tapped);

//       // if one of the columns is of type "Member", then filter out rows that requesting user is not a member of.
//       let has_member_column = b
//         .columns
//         .iter()
//         .any(|c| c.column_type == TableBlockColumnTypes::Member);

//       if has_member_column {
//         let member_col = b
//           .columns
//           .clone()
//           .into_iter()
//           .find(|c| c.column_type == TableBlockColumnTypes::Member)
//           .expect("member column not found");

//         let filtered_rows = b
//           .rows
//           .clone()
//           .into_iter()
//           .filter(|r| {
//             let member_cell = b
//               .cells
//               .clone()
//               .into_iter()
//               .find(|c| {
//                 c.cell_type == TableBlockCellTypes::Member
//                   && c.row_id == r.id
//                   && c.column_id == member_col.id
//               });

//             if let Some(m_cell) = member_cell {
//               if m_cell.cell_type == TableBlockCellTypes::Member {
//                 if let TableBlockCells::TableBlockMemberCell(cell) = m_cell.cell_data {
//                   cell
//                     .member_ids
//                     .contains(&user_id)
//                 } else {
//                   false
//                 }
//               } else {
//                 false
//               }
//             } else {
//               false
//             }
//           })
//           .collect::<Vec<TableBlockRow>>();

//         let filtered_row_ids = filtered_rows
//           .iter()
//           .map(|r| r.id)
//           .collect::<Vec<Uuid>>();

//         let filtered_cells = b
//           .cells
//           .clone()
//           .into_iter()
//           .filter(|c| filtered_row_ids.contains(&c.row_id))
//           .collect::<Vec<TableBlockCell>>();

//         let mut new_b = b.clone();
//         new_b.rows = filtered_rows;
//         new_b.cells = filtered_cells;

//         return Ok(serde_json::to_value(new_b)?);
//       } else {
//         Ok(block_data)
//       }
//     }
//     &_ => {
//       return Err(anyhow!("Block type not supported"));
//     }
//   };

//   a
// }

#[derive(Debug)]
pub struct BlockConnectionData {
  pub connection: DBConnection,
  pub source: Option<DBSource>,
  pub sourcequery: Option<DBSourceQuery>,
  pub blocks: Option<DBBlock>,
  // TODO: might want to add destination stuff to this struct.
}

pub fn get_block_source(source: DBSource) -> Result<Option<BlockSource>> {
  let bs = match source.source_type.as_str() {
    "Block" => {
      let block_source: BlockSource = serde_json::from_value(source.source_data)?;
      Some(block_source)
    }
    _ => None,
  };

  Ok(bs)
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

  // a "source" holds info about where the source data comes from. For now, the only source
  // type is "Block", which holds a block id.
  let source_ids = connections
    .clone()
    .into_iter()
    .filter_map(|c| c.source_id)
    .collect::<Vec<Uuid>>();
  let sources = get_sources(&mut tx, source_ids).await?;

  // Get source queries.
  let source_query_ids = connections
    .clone()
    .into_iter()
    .filter_map(|c| c.sourcequery_id)
    .collect::<Vec<Uuid>>();
  let sourcequeries = get_sourcequeries(&mut tx, source_query_ids).await?;

  let source_blocks = sources
    .clone()
    .into_iter()
    .filter_map(|s| get_block_source(s).expect("unable to get block source"))
    .collect::<Vec<BlockSource>>();

  // Get the Blocks that are referenced in the BlockSource objects.
  let source_block_ids = source_blocks
    .iter()
    .map(|bs| bs.block_id.clone())
    .collect::<Vec<Uuid>>();
  let db_blocks = get_blocks(&mut tx, source_block_ids).await?;

  let collected = connections
    .into_iter()
    .map(|db_connection| {
      let source = match db_connection.source_id {
        Some(id) => sources.iter().find(|s| s.id == id).cloned(),
        None => None,
      };

      let sourcequery = match db_connection.sourcequery_id {
        Some(id) => sourcequeries.iter().find(|sq| sq.id == id).cloned(),
        None => None,
      };

      let block_source = source.clone().map(|s| {
        get_block_source(s)
          .expect("unable to get block source")
          .unwrap() // TODO: handle this better
      });

      let bs_id = block_source.map(|bs| bs.block_id.clone());

      let db_block = match bs_id {
        Some(id) => db_blocks.iter().find(|b| b.id == id).cloned(),
        None => None,
      };

      BlockConnectionData {
        connection: db_connection,
        source: source,
        sourcequery: sourcequery,
        blocks: db_block, // should this be generalized?
      }
    })
    .collect::<Vec<BlockConnectionData>>();

  dbg!(&collected);

  let conn_content = collected
    .into_iter()
    .filter(|c| c.source.is_some())
    .map(|c| {
      let content_data = if let Some(db_block) = c.blocks {
        match c.sourcequery {
          // Source query found => Match on block_type to call correct query function
          // and prepare correct sourcequery args
          Some(sq) => match serde_json::from_value(sq.sourcequery_data) {
            Ok(GQLSourceQueries::TableBlock(q)) => {
              let block: Block = db_block.try_into()?;
              let table_block = if let GQLBlocks::Table(b) = block.block_data {
                b
              } else {
                return Err(anyhow!("Block is not of TableBlock type"));
              };
              // let sourcequery_args = construct_sourcequery_args(user.id);
              let queried_block =
                query_table_block(table_block, q, TableBlockQueryArgs { user_id: user.id })?;

              Ok(serde_json::to_string(&queried_block).map_or_else(|_| "[]".to_string(), |d| d))
            }
            Err(e) => Err(anyhow!(format!(
              "Failed to deserialize SourceQuery data: {}",
              e
            ))),
          },
          // No sourcequery to apply <=> identity transformation
          None => Ok(db_block.block_data.to_string()),
        }
      } else {
        Ok("[]".to_string())
      }?;

      Ok(ConnectionContent {
        source_id: c.source.unwrap().id,
        content_type: ContentTypes::TableBlock,
        content_data,
      })
    })
    .collect::<Result<Vec<ConnectionContent>>>()?;

  tx.commit().await?;

  Ok(conn_content)
}
