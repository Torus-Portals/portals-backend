use juniper::GraphQLObject;

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct GoogleSheetsDimension {
  pub empty: bool,
}
