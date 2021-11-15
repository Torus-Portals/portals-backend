use juniper::{GraphQLEnum, GraphQLObject, GraphQLUnion};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use crate::{services::db, utils::ir::*};

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct TableBlockRow {
  pub id: Uuid,

  pub index: i32,
}

impl From<TableBlockRow> for RowNode {
  fn from(row: TableBlockRow) -> RowNode {
    RowNode {
      id: row.id,
      index: row.index,
      label: format!("Row {}", row.index),
      debug_info: format!("TableBlockRow {}", row.id),
      data: None,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum TableBlockColumnTypes {
  #[strum(serialize = "Text")]
  #[graphql(name = "Text")]
  Text,

  #[strum(serialize = "Member")]
  #[graphql(name = "Member")]
  Member,
}

impl From<TableBlockColumnTypes> for ColumnNodeTypes {
  fn from(column_type: TableBlockColumnTypes) -> ColumnNodeTypes {
    match column_type {
      TableBlockColumnTypes::Text => ColumnNodeTypes::Text,
      TableBlockColumnTypes::Member => ColumnNodeTypes::Member,
    }
  }
}

// NOTE: In the future columns should have an access policy associated with them, in case
//       we want to restrict access to certain columns.
#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBlockColumn {
  pub id: Uuid,

  pub index: i32,

  pub column_type: TableBlockColumnTypes,

  pub label: String,
}

impl From<TableBlockColumn> for ColumnNode {
  fn from(column: TableBlockColumn) -> ColumnNode {
    ColumnNode {
      id: column.id,
      index: column.index,
      column_type: column
        .column_type
        .into(),
      label: column.label,
      debug_info: format!("TableBlockColumn {}", column.id),
      data: None,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum TableBlockCellTypes {
  #[strum(serialize = "Empty")]
  #[graphql(name = "Empty")]
  Empty,

  #[strum(serialize = "Text")]
  #[graphql(name = "Text")]
  Text,

  #[strum(serialize = "Member")]
  #[graphql(name = "Member")]
  Member,
}

impl From<TableBlockCellTypes> for CellNodeTypes {
  fn from(cell_type: TableBlockCellTypes) -> CellNodeTypes {
    match cell_type {
      TableBlockCellTypes::Empty => CellNodeTypes::Empty,
      TableBlockCellTypes::Text => CellNodeTypes::Text,
      TableBlockCellTypes::Member => CellNodeTypes::Member,
    }
  }
}

impl From<CellNodeTypes> for TableBlockCellTypes {
  fn from(cell_type: CellNodeTypes) -> TableBlockCellTypes {
    match cell_type {
      CellNodeTypes::Empty => TableBlockCellTypes::Empty,
      CellNodeTypes::Text => TableBlockCellTypes::Text,
      CellNodeTypes::Member => TableBlockCellTypes::Member,
    }
  }
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct TableBlockEmptyCell {
  pub id: Uuid,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct TableBlockTextCell {
  pub id: Uuid,

  pub text: String,
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBlockMemberCell {
  pub id: Uuid,

  // TODO: maybe a "all members may view" flag, or maybe an access policy.
  pub member_ids: Vec<Uuid>,
}

#[derive(GraphQLUnion, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TableBlockCells {
  TableBlockEmptyCell(TableBlockEmptyCell),
  TableBlockTextCell(TableBlockTextCell),
  TableBlockMemberCell(TableBlockMemberCell),
}

// impl From<TableBlockCells> for CellNodeTypes {
//   fn from(cell_type: TableBlockCells) -> CellNodeTypes {
//     match cell_type {
//       TableBlockCells::TableBlockEmptyCell(_) => CellNodeTypes::Empty,
//       TableBlockCells::TableBlockTextCell(_) => CellNodeTypes::Text,
//       TableBlockCells::TableBlockMemberCell(_) => CellNodeTypes::Member,
//     }
//   }
// }

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBlockCell {
  pub id: Uuid,

  pub row_id: Uuid,

  pub column_id: Uuid,

  pub cell_type: TableBlockCellTypes,

  pub cell_data: TableBlockCells,
}

impl From<TableBlockCell> for CellNode {
  fn from(cell: TableBlockCell) -> CellNode {
    let data = match cell.cell_data {
      TableBlockCells::TableBlockEmptyCell(_) => None,
      TableBlockCells::TableBlockTextCell(text) => Some(NodeTypes::Text(TextNode {
        id: cell.id,
        label: format!("Text for {}", cell.id),
        text: text.text,
      })),
      TableBlockCells::TableBlockMemberCell(member_cell) => {
        let nodes = member_cell
          .member_ids
          .into_iter()
          .map(|member_id| {
            NodeTypes::User(UserNode {
              id: member_id,
              label: format!("Member {}", member_id),
              user_id: member_id,
            })
          })
          .collect::<Vec<NodeTypes>>();

        Some(NodeTypes::List(ListNode {
          id: member_cell.id,
          label: format!("Member for {}", cell.id),
          nodes,
        }))
      }
    };

    CellNode {
      id: cell.id,
      row_id: cell.row_id,
      column_id: cell.column_id,
      cell_type: cell
        .cell_type
        .into(),
      label: format!("Cell {}", cell.id),
      debug_info: format!("TableBlockCell {}", cell.id),
      data,
    }
  }
}

#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct TableBlock {
  pub rows: Vec<TableBlockRow>,

  pub columns: Vec<TableBlockColumn>,

  pub cells: Vec<TableBlockCell>,
}

impl From<TableBlock> for TableNode {
  fn from(table: TableBlock) -> TableNode {
    let rows = table
      .rows
      .into_iter()
      .map(|row| row.into())
      .collect::<Vec<RowNode>>();

    let columns = table
      .columns
      .into_iter()
      .map(|column| column.into())
      .collect::<Vec<ColumnNode>>();

    let cells = table
      .cells
      .into_iter()
      .map(|cell| cell.into())
      .collect::<Vec<CellNode>>();

    TableNode {
      id: Uuid::new_v4(),
      label: "Table".to_string(),
      debug_info: "TableBlock".to_string(),
      rows,
      columns,
      cells,
    }
  }
}

impl IRTap for TableBlock {
  fn tap(&self) -> Node {
    let table_node: TableNode = self
      .to_owned()
      .into();

    Node {
      id: Uuid::new_v4(),
      label: "Root Node".to_string(),
      debug_info: format!("Root Node for"),
      data: Some(NodeTypes::Table(table_node)),
    }
  }
}

#[derive(Debug, Clone)]
pub struct TableBlockVisitor {
  pub current_cell: Option<TableBlockCell>,

  pub rows: Vec<TableBlockRow>,

  pub columns: Vec<TableBlockColumn>,

  pub cells: Vec<TableBlockCell>,
}

impl TableBlockVisitor {
  fn new() -> Self {
    TableBlockVisitor {
      current_cell: None,
      rows: vec![],
      columns: vec![],
      cells: Vec::new(),
    }
  }
}

impl IRVisitors for TableBlockVisitor {
  fn enter_row(&mut self, row: &RowNode, _state: &IRVisitorState) {
    dbg!(&row);

    self
      .rows
      .push(TableBlockRow {
        id: row.id,
        index: row.index,
      });
  }

  fn enter_column(&mut self, column: &ColumnNode, _state: &IRVisitorState) {
    let column_type = match column.column_type {
      ColumnNodeTypes::Text => TableBlockColumnTypes::Text,
      ColumnNodeTypes::Member => TableBlockColumnTypes::Member,
    };

    self
      .columns
      .push(TableBlockColumn {
        id: column.id,
        index: column.index,
        column_type,
        label: column
          .label
          .to_owned(),
      });
  }

  fn enter_cell(&mut self, cell_node: &CellNode, _state: &IRVisitorState) {
    let cell = TableBlockCell {
      id: cell_node.id,
      row_id: cell_node.row_id,
      column_id: cell_node.column_id,
      cell_type: cell_node
        .cell_type
        .clone()
        .into(),
      cell_data: TableBlockCells::TableBlockEmptyCell(TableBlockEmptyCell { id: cell_node.id }),
    };

    self.current_cell = Some(cell);
  }

  fn exit_cell(&mut self, _cell_node: &CellNode, _state: &IRVisitorState) {
    if let Some(cell) = self.current_cell.take() {
      self.cells.push(cell);
    };
  }

  fn enter_text(&mut self, text_node: &TextNode, _state: &IRVisitorState) {
    if let Some(cell) = self.current_cell.as_mut() {
      cell.cell_data = TableBlockCells::TableBlockTextCell(TableBlockTextCell {
        id: text_node.id,
        text: text_node.text.to_owned(),
      });
    }
  }

  fn enter_user(&mut self, user: &UserNode, _state: &IRVisitorState) {
    println!("enter_user");

    if let Some(cell) = self.current_cell.as_mut() {
      match &mut cell.cell_data {
        TableBlockCells::TableBlockEmptyCell(_) => {
          cell.cell_data = TableBlockCells::TableBlockMemberCell(TableBlockMemberCell {
            id: cell.id,
            member_ids: vec![user.user_id],
          });
        },
        TableBlockCells::TableBlockMemberCell(member_cell) => {
          member_cell.member_ids.push(user.user_id);
        },
        _ => todo!(),
    }
    };
  }
}

impl IRSink for TableBlock {
  fn sink(&self, root_node: Node) -> Self {
    dbg!(&root_node);
    // let cloned_root_node = root_node.clone();
    let state = IRVisitorState::new(root_node.clone());
    let visitor = TableBlockVisitor::new();
    let a = visit(root_node, visitor, state);

    dbg!(&a.0.rows);
    dbg!(&a.0.columns);
    dbg!(&a.0.cells);

    TableBlock {
      rows: vec![],
      columns: vec![],
      cells: vec![],
    }
  }
}
