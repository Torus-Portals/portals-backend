use std::collections::HashMap;
use juniper::{GraphQLEnum, GraphQLObject, GraphQLUnion};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use crate::graphql::context::GQLContext;
use crate::utils::ir::*;

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum XYChartDatumTypes {
  #[strum(serialize = "Text")]
  #[graphql(name = "Text")]
  Text,

  #[strum(serialize = "Number")]
  #[graphql(name = "Number")]
  Number,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
#[serde(rename_all = "camelCase")]
pub struct XYChartTextDatum {
  key: String,
  datum_type: XYChartDatumTypes,
  text: String,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
#[serde(rename_all = "camelCase")]
pub struct XYChartNumberDatum {
  key: String,
  datum_type: XYChartDatumTypes,
  number: f64,
}

#[derive(Debug, Clone, GraphQLUnion, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub enum XYChartDatum {
  Text(XYChartTextDatum),
  Number(XYChartNumberDatum),
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub struct XYChartDatumGroup {
  pub id: Uuid, // probably row id
  pub data: Vec<XYChartDatum>,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub struct XYChartSeries {
  pub id: Uuid, // Table id, I think.

  pub label: String,

  pub groups: Vec<XYChartDatumGroup>,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
#[serde(rename_all = "camelCase")]
pub struct XYChartBlock {
  chart_type: XYChartTypes,

  primary_axis: Option<Uuid>,

  series: Vec<XYChartSeries>,
}

impl From<XYChartBlock> for TableNode {
  fn from(_block: XYChartBlock) -> TableNode {
    TableNode {
      id: Uuid::new_v4(),
      label: "Table".to_string(),
      debug_info: "XYChartBlock".to_string(),
      rows: vec![],
      columns: vec![],
      cells: vec![],
    }
  }
}

impl IRTap for XYChartBlock {
  fn tap(&self) -> Node {
    Node {
      id: Uuid::new_v4(),
      label: "Root Node".to_string(),
      debug_info: "Root Node for XYChart".to_string(),
      data: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct XYChartBlockVisitor {
  chart_type: XYChartTypes,

  primary_axis: Option<Uuid>,

  current_series: Option<Uuid>,

  series: Vec<XYChartSeries>,

  column_names_by_column_id: HashMap<Uuid, String>,
}

impl XYChartBlockVisitor {
  fn new() -> Self {
    XYChartBlockVisitor {
      chart_type: XYChartTypes::Line,
      primary_axis: None,
      current_series: None,
      series: vec![],
      column_names_by_column_id: HashMap::new(),
    }
  }
}

impl IRVisitors for XYChartBlockVisitor {
  fn enter_table(&mut self, table: &TableNode, _state: &IRVisitorState) {
    let series = XYChartSeries {
      id: table.id,
      label: "Series 1".to_string(),
      groups: vec![],
    };

    self
      .series
      .push(series);

    self.current_series = Some(table.id.clone());
  }

  fn enter_column(&mut self, column: &ColumnNode, _state: &IRVisitorState) {
    self
      .column_names_by_column_id
      .insert(column.id, column.label.clone());
  }

  fn enter_row(&mut self, row: &RowNode, _state: &IRVisitorState) {
    let current_series_id = self
      .current_series
      .unwrap_or_else(|| Uuid::new_v4());

    if let Some(current_series) = self
      .series
      .iter_mut()
      .find(|s| s.id == current_series_id)
    {
      current_series
        .groups
        .push(XYChartDatumGroup {
          id: row.id.clone(),
          data: vec![],
        });
    }
  }

  fn enter_cell(&mut self, cell: &CellNode, _state: &IRVisitorState) {
    let current_series_id = self
      .current_series
      .unwrap_or_else(|| Uuid::new_v4());

    if let Some(current_series) = self
      .series
      .iter_mut()
      .find(|s| s.id == current_series_id)
    {
      // find the right series group
      if let Some(series_group) = current_series
        .groups
        .iter_mut()
        .find(|g| g.id == cell.row_id)
      {
        let column_name = self
          .column_names_by_column_id
          .get(&cell.column_id)
          .unwrap()
          .to_owned();
        // .unwrap_or_else(|| "".to_string().as_str());

        if let Some(cell_data) = cell.data.clone() {
          let datum = match cell_data {
            NodeTypes::Text(text_node) => XYChartDatum::Text(XYChartTextDatum {
              key: column_name.clone(),
              datum_type: XYChartDatumTypes::Text,
              text: text_node
                .text
                .clone(),
            }),
            NodeTypes::Float(float_node) => XYChartDatum::Number(XYChartNumberDatum {
              key: column_name.clone(),
              datum_type: XYChartDatumTypes::Number,
              number: float_node.value,
            }),
            NodeTypes::User(user_node) => XYChartDatum::Text(XYChartTextDatum {
              key: column_name.clone(),
              datum_type: XYChartDatumTypes::Text,
              text: user_node
                .label
                .clone(),
            }),
            _ => XYChartDatum::Text(XYChartTextDatum {
              key: column_name.clone(),
              datum_type: XYChartDatumTypes::Text,
              text: "".to_string(),
            }),
          };

          series_group
            .data
            .push(datum);
        }
      }
    }
  }
}

impl IRSink for XYChartBlock {
  fn sink(root_node: Node) -> Self {
    let state = IRVisitorState::new(root_node.clone());
    let visitor = XYChartBlockVisitor::new();
    let a = visit(root_node, visitor, state);

    XYChartBlock {
      chart_type: a.0.chart_type,
      primary_axis: None,
      series: a.0.series,
    }
  }
}
