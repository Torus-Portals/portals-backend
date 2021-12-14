use juniper::{GraphQLEnum, GraphQLObject, GraphQLUnion};
use uuid::Uuid;

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct FilesBlock {
  pub id: Uuid,

  pub files: Vec<Uuid>,
}