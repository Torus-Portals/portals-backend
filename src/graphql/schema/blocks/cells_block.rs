use juniper::GraphQLObject;
use uuid::Uuid;

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct CellsBlock {
  pub id: Uuid,
}
