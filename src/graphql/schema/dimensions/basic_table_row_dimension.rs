use juniper::{GraphQLObject};

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct BasicTableRowDimension {
  pub empty: bool
}