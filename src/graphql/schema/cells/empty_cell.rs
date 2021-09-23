use juniper::GraphQLObject;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct EmptyCell {
  pub cell_type: String,
}