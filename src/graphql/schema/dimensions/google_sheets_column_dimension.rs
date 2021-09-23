use juniper::GraphQLObject;

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct GoogleSheetsColumnDimension {
  pub empty: bool,
}
