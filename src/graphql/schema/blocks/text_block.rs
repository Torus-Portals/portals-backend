use juniper::{GraphQLObject};
use uuid::Uuid;

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct TextBlock {
    pub id: Uuid,

    pub text: String,
}