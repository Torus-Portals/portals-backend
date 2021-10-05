use juniper::GraphQLObject;

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct GoogleSheetsRowDimension {
  pub empty: bool,
}
