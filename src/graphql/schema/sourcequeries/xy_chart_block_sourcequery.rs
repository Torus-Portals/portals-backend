use juniper::GraphQLObject;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct XYChartBlockSourceQuery {
  pub todo: bool,
}