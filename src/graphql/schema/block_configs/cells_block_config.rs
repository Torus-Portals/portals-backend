use juniper::{GraphQLObject};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct CellsBlockConfig {
  something: Option<String>,
}
