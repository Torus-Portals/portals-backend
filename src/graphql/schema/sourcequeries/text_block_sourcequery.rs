use juniper::GraphQLObject;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct TextBlockSourceQuery {
  pub todo: bool,
}