use std::str::FromStr;

use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLEnum, GraphQLObject, GraphQLUnion};
use strum_macros::EnumString;

// use juniper::{GraphQLInputObject};

// use super::Mutation;
use super::Query;

use crate::graphql::context::GQLContext;
use crate::services::db::cell_service::{DBCell, get_cell};
use uuid::Uuid;

#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
pub enum GQLCells {
  BasicText(BasicTextCell),
  Empty(EmptyCell),
}

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString)]
pub enum CellTypes {
  BasicText,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct Cell {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  // NOTE: Should probably be good to make this an enum.
  #[serde(rename = "cellType")]
  pub cell_type: CellTypes,

  pub dimensions: Vec<Uuid>,

  #[serde(rename = "cellData")]
  pub cell_data: GQLCells,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

impl From<DBCell> for Cell {
  fn from(db_cell: DBCell) -> Self {
    let cell_data = match db_cell.cell_type.as_str() {
      "BasicText" => {
        let c: BasicTextCell =
          serde_json::from_value(db_cell.data).expect("Can't deserialize BasicTextCell");
        GQLCells::BasicText(c)
      }
      &_ => GQLCells::Empty(EmptyCell {
        cell_type: String::from("nothing"),
      }),
    };

    let cell_type = CellTypes::from_str(db_cell.cell_type.as_str())
      .expect("Unable to convert cell_type string to enum variant");

    Cell {
      id: db_cell.id,
      portal_id: db_cell.portal_id,
      cell_type,
      dimensions: db_cell.dimensions,
      cell_data,
      created_at: db_cell.created_at,
      created_by: db_cell.created_by,
      updated_at: db_cell.updated_at,
      updated_by: db_cell.updated_by,
    }
  }
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct BasicTextCell {
  text: String,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct EmptyCell {
  cell_type: String,
}

impl Query {
  pub async fn cell_impl(ctx: &GQLContext, cell_id: Uuid) -> FieldResult<Cell> {
      get_cell(&ctx.pool, cell_id)
      .await
      .map(|db_cell| db_cell.into())
      .map_err(FieldError::from)
  }
}
