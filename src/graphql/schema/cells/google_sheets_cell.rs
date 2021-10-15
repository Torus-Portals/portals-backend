use juniper::GraphQLObject;
use uuid::Uuid;

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct GoogleSheetsCell {
  // TODO: figure out what values should be stored in this cell
  pub integration_id: Uuid,

  // pub row_dimension: Uuid,

  // pub col_dimension: Uuid,

  pub value: String,
}