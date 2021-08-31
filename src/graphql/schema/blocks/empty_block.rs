use juniper::{GraphQLObject};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct EmptyBlock {
  pub block_type: String,
}