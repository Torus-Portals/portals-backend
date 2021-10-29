use juniper::GraphQLObject;
use uuid::Uuid;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockSource {
  pub block_id: Uuid,
}

impl From<serde_json::Value> for BlockSource {
  fn from(value: serde_json::Value) -> Self {
    serde_json::from_value::<BlockSource>(value)
    .expect("Failed to convert json into BlockSource")
  }
}
