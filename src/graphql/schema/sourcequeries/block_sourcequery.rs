use juniper::{GraphQLObject};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct BlockSourceQuery {
  pub block_type: String,
}