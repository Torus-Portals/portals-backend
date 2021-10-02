use juniper::{GraphQLObject};
use uuid::Uuid;

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct PortalMemberDimension {
  #[serde(rename = "userId")]
  pub user_id: Uuid
}