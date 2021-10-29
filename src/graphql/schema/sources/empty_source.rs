use juniper::{GraphQLObject};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmptySource {
  pub empty: bool,
}

impl From<serde_json::Value> for EmptySource {
  fn from(value: serde_json::Value) -> Self {
    serde_json::from_value::<EmptySource>(value)
    .expect("Failed to convert json into EmptySource")
  }
}