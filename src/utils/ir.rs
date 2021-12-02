use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EmptyNode {}

#[derive(Debug, Clone)]
pub struct TextNode {
  pub id: Uuid,
  pub label: String,
  pub text: String,
}

#[derive(Debug, Clone)]
pub struct IntNode {
  pub id: Uuid,
  pub label: String,
  pub value: i32,
}

#[derive(Debug, Clone)]
pub struct FloatNode {
  pub id: Uuid,
  pub label: String,
  pub value: f64,
}

#[derive(Debug, Clone)]
pub struct UuidNode {
  pub id: Uuid,
  pub label: String,
  pub uuid: Uuid,
}

#[derive(Debug, Clone)]
pub struct UserNode {
  pub id: Uuid,
  pub label: String,
  pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ListNode {
  pub id: Uuid,
  pub label: String,
  pub nodes: Vec<NodeTypes>,
}

#[derive(Debug, Clone)]
pub struct TableNode {
  pub id: Uuid,
  pub label: String,
  pub debug_info: String,
  pub rows: Vec<RowNode>,
  pub columns: Vec<ColumnNode>,
  pub cells: Vec<CellNode>,
}

#[derive(Debug, Clone)]
pub struct RowNode {
  pub id: Uuid,
  pub index: i32,
  pub label: String,
  pub debug_info: String,
  pub data: Option<NodeTypes>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColumnNodeTypes {
  Text,
  Number,
  Member,
}

#[derive(Debug, Clone)]
pub struct ColumnNode {
  pub id: Uuid,
  pub index: i32,
  pub column_type: ColumnNodeTypes,
  pub label: String,
  pub debug_info: String,
  pub data: Option<NodeTypes>,
}

#[derive(Debug, Clone)]
pub enum CellNodeTypes {
  Empty,
  Text,
  Number,
  Member,
}

#[derive(Debug, Clone)]
pub struct CellNode {
  pub id: Uuid,
  pub row_id: Uuid,
  pub column_id: Uuid,
  pub cell_type: CellNodeTypes,
  pub label: String,
  pub debug_info: String,
  pub data: Option<NodeTypes>,
}

#[derive(Debug, Clone)]
pub enum NodeTypes {
  // Empty(EmptyNode),
  // Node(Box<Node>),
  List(ListNode),
  Table(TableNode),
  Row(Box<RowNode>),
  Column(Box<ColumnNode>),
  Cell(Box<CellNode>),
  Text(TextNode),
  // Int(IntNode),
  Float(FloatNode),
  // Uuid(UuidNode),
  User(UserNode),
}

#[derive(Debug, Clone)]
pub struct Node {
  pub id: Uuid,
  pub label: String,
  pub debug_info: String,
  pub data: Option<NodeTypes>,
}

pub trait IRTap {
  fn tap(&self) -> Node;
}

pub trait IRSink {
  fn sink(&self, root_node: Node) -> Self;
}

#[derive(Debug, Clone)]
pub struct IRVisitorState {
  pub root_node: Node,
  pub level: i32,
}

impl IRVisitorState {
  pub fn new(root_node: Node) -> Self {
    Self {
      root_node,
      level: 0,
    }
  }
}

pub trait IRVisitors {
  fn enter_node(&mut self, _node: &Node, _state: &IRVisitorState) {}
  fn exit_node(&mut self, _node: &Node, _state: &IRVisitorState) {}

  fn enter_list(&mut self, _list: &ListNode, _state: &IRVisitorState) {}
  fn exit_list(&mut self, _list: &ListNode, _state: &IRVisitorState) {}

  fn enter_table(&mut self, _table: &TableNode, _state: &IRVisitorState) {}
  fn exit_table(&mut self, _table: &TableNode, _state: &IRVisitorState) {}

  fn enter_row(&mut self, _row: &RowNode, _state: &IRVisitorState) {}
  fn exit_row(&mut self, _row: &RowNode, _state: &IRVisitorState) {}

  fn enter_column(&mut self, _column: &ColumnNode, _state: &IRVisitorState) {}
  fn exit_column(&mut self, _column: &ColumnNode, _state: &IRVisitorState) {}

  fn enter_cell(&mut self, _cell: &CellNode, _state: &IRVisitorState) {}
  fn exit_cell(&mut self, _cell: &CellNode, _state: &IRVisitorState) {}

  fn enter_empty(&mut self, _empty: &EmptyNode, _state: &IRVisitorState) {}
  fn exit_empty(&mut self, _empty: &EmptyNode, _state: &IRVisitorState) {}

  fn enter_text(&mut self, _text: &TextNode, _state: &IRVisitorState) {}
  fn exit_text(&mut self, _text: &TextNode, _state: &IRVisitorState) {}

  fn enter_int(&mut self, _int: &IntNode, _state: &IRVisitorState) {}
  fn exit_int(&mut self, _int: &IntNode, _state: &IRVisitorState) {}

  fn enter_float(&mut self, _float: &FloatNode, _state: &IRVisitorState) {}
  fn exit_float(&mut self, _float: &FloatNode, _state: &IRVisitorState) {}

  fn enter_uuid(&mut self, _uuid: &UuidNode, _state: &IRVisitorState) {}
  fn exit_uuid(&mut self, _uuid: &UuidNode, _state: &IRVisitorState) {}

  fn enter_user(&mut self, _user: &UserNode, _state: &IRVisitorState) {}
  fn exit_user(&mut self, _user: &UserNode, _state: &IRVisitorState) {}
}

pub fn visit<IRV: IRVisitors + Clone>(
  node: Node,
  mut visitor: IRV,
  mut state: IRVisitorState,
) -> (IRV, IRVisitorState) {
  state.level += 1;

  /* */
  visitor.enter_node(&node, &state);
  /* */

  let mut v_and_s = match node.data.as_ref() {
    Some(node_type) => visit_node_type(node_type, visitor, state),
    None => (visitor, state),
  };

  v_and_s
    .0
    .exit_node(&node, &v_and_s.1);

  v_and_s.1.level -= 1;

  v_and_s
}

pub fn visit_node_type<IRV: IRVisitors + Clone>(
  node_type: &NodeTypes,
  mut visitor: IRV,
  state: IRVisitorState,
) -> (IRV, IRVisitorState) {
  match node_type {
    // NodeTypes::Node(n) => {
    //   // dbg!(&n);
    //   let unboxed_n = *(n.clone());

    //   // visitor.enter_node(&unboxed_node, &state);
    //   let next = visit(unboxed_n, visitor, state);

    //   // next.0.exit_node(&n, &next.1);

    //   next
    // }
    NodeTypes::List(list_node) => {
      visitor.enter_list(list_node, &state);

      let mut next = list_node
        .nodes
        .clone()
        .into_iter()
        .fold((visitor, state), |(v, s), n| visit_node_type(&n, v, s));

      // need to iterate over all the nodes in the list.
      next
        .0
        .exit_list(list_node, &next.1);

      next
    }
    NodeTypes::Table(table_node) => {
      visitor.enter_table(table_node, &state);

      let next = table_node
        .rows
        .clone()
        .into_iter()
        .fold((visitor, state), |(v, s), n| {
          visit_node_type(&NodeTypes::Row(Box::new(n)), v, s)
        });

      let next = table_node
        .columns
        .clone()
        .into_iter()
        .fold(next, |(v, s), n| {
          visit_node_type(&NodeTypes::Column(Box::new(n)), v, s)
        });

      let mut next = table_node
        .cells
        .clone()
        .into_iter()
        .fold(next, |(v, s), n| {
          visit_node_type(&NodeTypes::Cell(Box::new(n)), v, s)
        });

      // need to iterate over all the nodes in the list.
      next
        .0
        .exit_table(table_node, &next.1);

      next
    }
    NodeTypes::Row(row_node) => {
      visitor.enter_row(row_node, &state);
      visitor.exit_row(row_node, &state);

      (visitor, state)
    }
    NodeTypes::Column(column_node) => {
      visitor.enter_column(column_node, &state);
      visitor.exit_column(column_node, &state);

      (visitor, state)
    }
    NodeTypes::Cell(cell_node) => {
      visitor.enter_cell(cell_node, &state);

      let mut next = match cell_node
        .data
        .as_ref()
      {
        Some(node_type) => visit_node_type(node_type, visitor, state),
        None => (visitor, state),
      };

      next
        .0
        .exit_cell(cell_node, &next.1);

      next
    }
    // NodeTypes::Empty(empty_node) => {
    //   visitor.enter_empty(empty_node, &state);
    //   visitor.exit_empty(empty_node, &state);

    //   (visitor, state)
    // }
    NodeTypes::Text(text_node) => {
      visitor.enter_text(text_node, &state);
      visitor.exit_text(text_node, &state);

      (visitor, state)
    }
    // NodeTypes::Int(int_node) => {
    //   visitor.enter_int(int_node, &state);
    //   visitor.exit_int(int_node, &state);

    //   (visitor, state)
    // }
    NodeTypes::Float(float_node) => {
      visitor.enter_float(float_node, &state);
      visitor.exit_float(float_node, &state);

      (visitor, state)
    }
    // NodeTypes::Uuid(uuid_node) => {
    //   visitor.enter_uuid(uuid_node, &state);
    //   visitor.exit_uuid(uuid_node, &state);

    //   (visitor, state)
    // }
    NodeTypes::User(user_node) => {
      visitor.enter_user(user_node, &state);
      visitor.exit_user(user_node, &state);

      (visitor, state)
    }
  }
}
