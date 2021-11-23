use juniper::{GraphQLEnum, GraphQLObject, GraphQLUnion};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct CellsBlock {
    pub id: Uuid,
}