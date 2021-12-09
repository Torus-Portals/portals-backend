use anyhow::{anyhow, Result};
use juniper::GraphQLObject;
use uuid::Uuid;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct CellsBlockSourceQuery {
  pub todo: bool,
}