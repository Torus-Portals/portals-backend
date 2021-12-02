use juniper::{GraphQLObject};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct TableBlockConfig {
  something: Option<String>,
}
