use juniper::{GraphQLObject};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct EmptyBlockConfig {
  empty: bool,
}
