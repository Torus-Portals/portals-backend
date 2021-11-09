use juniper::{GraphQLEnum, GraphQLObject, GraphQLUnion};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct TableBlockRow {
  pub id: Uuid,

  pub index: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum TableBlockColumnTypes {
  #[strum(serialize = "Text")]
  #[graphql(name = "Text")]
  Text,

  #[strum(serialize = "Member")]
  #[graphql(name = "Member")]
  Member,
}

// NOTE: In the future columns should have an access policy associated with them, in case
//       we want to restrict access to certain columns.
#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBlockColumn {
  pub id: Uuid,

  pub column_type: TableBlockColumnTypes,

  pub label: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum TableBlockCellTypes {
  #[strum(serialize = "Empty")]
  #[graphql(name = "Empty")]
  Empty,

  #[strum(serialize = "Text")]
  #[graphql(name = "Text")]
  Text,

  #[strum(serialize = "Member")]
  #[graphql(name = "Member")]
  Member,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct TableBlockEmptyCell {
  pub id: Uuid,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct TableBlockTextCell {
  pub id: Uuid,

  pub text: String,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBlockMemberCell {
  pub id: Uuid,

  // TODO: maybe a "all members may view" flag, or maybe an access policy.
  pub member_ids: Vec<Uuid>,
}

#[derive(GraphQLUnion, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TableBlockCells {
  TableBlockEmptyCell(TableBlockEmptyCell),
  TableBlockTextCell(TableBlockTextCell),
  TableBlockMemberCell(TableBlockMemberCell),
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBlockCell {
  pub id: Uuid,

  pub row_id: Uuid,

  pub column_id: Uuid,

  pub cell_type: TableBlockCellTypes,

  pub cell_data: TableBlockCells,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct TableBlock {
  pub rows: Vec<TableBlockRow>,

  pub columns: Vec<TableBlockColumn>,

  pub cells: Vec<TableBlockCell>,
}
