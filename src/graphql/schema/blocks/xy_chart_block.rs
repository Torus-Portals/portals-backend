use juniper::{GraphQLEnum, GraphQLObject};
use strum_macros::{Display, EnumString};

// For now, basing this on how ReactCharts works.
// https://react-charts.tanstack.com/docs/api

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum XYChartTypes {
  #[strum(serialize = "Line")]
  #[graphql(name = "Line")]
  Line,

  #[strum(serialize = "Bar")]
  #[graphql(name = "Bar")]
  Bar,

  #[strum(serialize = "StackedBar")]
  #[graphql(name = "StackedBar")]
  StackedBar,

  #[strum(serialize = "HorizontalBar")]
  #[graphql(name = "HorizontalBar")]
  HorizontalBar,

  #[strum(serialize = "HorizontalStackedBar")]
  #[graphql(name = "HorizontalStackedBar")]
  HorizontalStackedBar,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum XYChartAxisTypes {
  #[strum(serialize = "Linear")]
  #[graphql(name = "Linear")]
  Linear,

  #[strum(serialize = "Band")]
  #[graphql(name = "Band")]
  Band,

  #[strum(serialize = "Time")]
  #[graphql(name = "Time")]
  Time,
  
  #[strum(serialize = "TimeLocal")]
  #[graphql(name = "TimeLocal")]
  TimeLocal,

  #[strum(serialize = "Log")]
  #[graphql(name = "Log")]
  Log,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct XYChartAxis {
  pub label: String,
  pub values: Vec<String>,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XYChartBlock {
  chart_type: XYChartTypes,

  primary_axis: XYChartAxis,

  secondary_axis: Vec<XYChartAxis>,
}