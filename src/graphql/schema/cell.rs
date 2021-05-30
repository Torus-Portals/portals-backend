use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult};

use juniper::{GraphQLInputObject, GraphQLObject};

use super::Mutation;
use super::Query;

use crate::graphql::context::GQLContext;
use crate::services::db::cell_service::{DBCell};
use uuid::Uuid;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct Cell {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  // NOTE: Should probably be good to make this an enum.
  #[serde(rename = "cellType")]
  pub cell_type: String,

  pub dimensions: Vec<Uuid>,

  pub data: serde_json::Value,

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
    User {
      id: db_cell.id,
      portal_id: db_cell.portal_id,
      cell_type: db_cell.cell_type,
      dimensions: db_cell.dimensions,
      data: db_cell.data,
      created_at: db_cell.created_at,
      created_by: db_cell.created_by,
      updated_at: db_cell.updated_at,
      updated_by: db_cell.updated_by,
    }
  }
}

impl Query {
  pub async fn cell_impl(ctx: &GQLContext, cell_id: Uuid) -> FieldResult<Cell> {
    ctx
      .db
      .get_cell(cell_id)
      .await
      .map(|db_cell| db_cell.into())
      .map_err(FieldError::from)
  }
}