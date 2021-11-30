use juniper::{GraphQLObject};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct XYChartBlockConfig {
  something: Option<String>,
}
