use juniper::{GraphQLObject};

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct XYChartBlockConfig {
  something: Option<String>,
}
