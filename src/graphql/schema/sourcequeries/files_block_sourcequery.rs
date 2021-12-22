use juniper::GraphQLObject;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct FilesBlockSourceQuery {
  pub todo: bool,
}