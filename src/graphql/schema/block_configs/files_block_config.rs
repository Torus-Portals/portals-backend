use juniper::{GraphQLObject};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct FilesBlockConfig {
  something: Option<String>,
}
