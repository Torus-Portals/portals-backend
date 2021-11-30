use juniper::{GraphQLObject};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct TextBlockConfig {
  something: Option<String>,
}
