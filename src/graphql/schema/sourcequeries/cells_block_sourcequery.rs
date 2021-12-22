use juniper::GraphQLObject;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct CellsBlockSourceQuery {
  pub todo: bool,
}