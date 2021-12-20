use anyhow::{anyhow, Result};
use juniper::GraphQLObject;
use uuid::Uuid;

use crate::graphql::schema::{blocks::table_block::{
  TableBlock, TableBlockCell, TableBlockCells, TableBlockColumnTypes, TableBlockRow,
}, connection_content::SourceQueryArgs};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct TableBlockSourceQuery {
  pub member: bool,
}

pub fn filter_table_block_member(
  table_block: TableBlock,
  sourcequery_args: SourceQueryArgs,
) -> Result<TableBlock> {
  // Extract the member column id
  // TODO: which member column to use should be configured via the query args.
  let member_column_id = table_block
    .columns
    .iter()
    .find(|col| matches!(col.column_type, TableBlockColumnTypes::Member))
    .map_or_else(|| Err(anyhow!("Member column not found")), |col| Ok(col.id))?;

  // Extract the cells in the member column
  let member_cells = table_block
    .cells
    .iter()
    .filter(|c| c.column_id == member_column_id)
    .collect::<Vec<&TableBlockCell>>();

  // Extract the ids of rows which this user has access to
  let user_filtered_row_ids = member_cells
    .into_iter()
    .filter(|c| match &c.cell_data {
      TableBlockCells::TableBlockMemberCell(mc) => mc.member_ids.contains(&sourcequery_args.user_id),
      _ => false,
    })
    .map(|c| c.row_id)
    .collect::<Vec<Uuid>>();

  // Extract the rows this user has access to
  let user_rows = table_block
    .rows
    .iter()
    .filter(|row| user_filtered_row_ids.contains(&row.id))
    .cloned()
    .collect::<Vec<TableBlockRow>>();
  let user_columns = table_block.columns.clone();

  // Get all cells the user has access to
  let user_filtered_cells = table_block
    .cells
    .into_iter()
    .filter(|c| user_filtered_row_ids.contains(&c.row_id))
    .collect::<Vec<TableBlockCell>>();

  Ok(TableBlock {
    rows: user_rows,
    columns: user_columns,
    cells: user_filtered_cells,
  })
}

pub fn query_table_block(
  table_block: TableBlock,
  sourcequery: TableBlockSourceQuery,
  sourcequery_args: SourceQueryArgs,
) -> Result<TableBlock> {
  let TableBlockSourceQuery { member } = sourcequery;
  let mut final_table_block = table_block;

  if member {
    final_table_block = filter_table_block_member(final_table_block, sourcequery_args)?;
  }

  Ok(final_table_block)
}
