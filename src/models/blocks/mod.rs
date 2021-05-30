#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
pub enum Blocks {
  BasicTableBlock(BasicTableBlock)
}

pub fn block_data_from_serde_val(s_val: serde_json::Value) -> Option<Blocks> {
  match s_val["block_type"] {
    
    _ => None,
  }
}