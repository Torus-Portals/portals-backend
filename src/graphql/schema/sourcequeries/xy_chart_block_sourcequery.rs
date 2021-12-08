use anyhow::{anyhow, Result};
use juniper::GraphQLObject;
use uuid::Uuid;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct XYChartBlockSourceQuery {
  pub todo: bool,
}