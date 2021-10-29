use juniper::{ GraphQLObject};
use uuid::Uuid;

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct TableBlockRow {
  pub id: Uuid,
  pub index: i32,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct TableBlockColumn {
  pub id: Uuid,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBlockCell {
  pub id: Uuid,
  pub row_id: Uuid,
  pub column_id: Uuid,
  pub value: String,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct TableBlock {
  pub rows: Vec<TableBlockRow>,

  pub columns: Vec<TableBlockColumn>,

  pub cells: Vec<TableBlockCell>,
}


