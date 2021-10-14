use juniper::{GraphQLObject};

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct BasicTableColumnDimension {
  pub empty: bool
}