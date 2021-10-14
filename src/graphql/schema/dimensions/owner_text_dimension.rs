use juniper::{GraphQLObject};

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct OwnerTextDimension {
  pub empty: bool
}