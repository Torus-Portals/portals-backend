use juniper::{GraphQLObject};

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct EmptyDimension {
  pub empty: bool,
}
