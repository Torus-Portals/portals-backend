use std::str::FromStr;

use chrono::{DateTime, Utc};
use juniper::graphql_object;
use juniper::{
  FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, GraphQLUnion,
};
use strum_macros::{Display, EnumString};

use super::cells::basic_text_cell::BasicTextCell;
use super::cells::empty_cell::EmptyCell;
use super::cells::google_sheets_cell::GoogleSheetsCell;
use super::cells::owner_text_cell::OwnerTextCell;
use super::dimension::Dimension;
use super::integration::{Integration, IntegrationData};
use super::Mutation;
use super::Query;

use crate::graphql::context::GQLContext;
use crate::services::db::cell_service::{
  get_cell, get_cells_with_all_dimensions, get_cells_with_any_dimensions, create_cell, update_cell, DBCell,
};
use crate::services::db::dimension_service::get_dimension;
use crate::services::db::integration_service::get_integration;
use uuid::Uuid;

#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
pub enum GQLCells {
  BasicTextCell(BasicTextCell),
  OwnerTextCell(OwnerTextCell),
  GoogleSheetsCell(GoogleSheetsCell),
  Empty(EmptyCell),
}

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]

pub enum CellTypes {
  #[strum(serialize = "BasicText")]
  #[graphql(name = "BasicText")]
  BasicText,

  #[strum(serialize = "OwnerText")]
  #[graphql(name = "OwnerText")]
  OwnerText,

  #[strum(serialize = "GoogleSheets")]
  #[graphql(name = "GoogleSheets")]
  GoogleSheets,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
  pub id: Uuid,

  pub portal_id: Uuid,

  // NOTE: Should probably be good to make this an enum.
  pub cell_type: CellTypes,

  pub dimensions: Vec<Uuid>,

  pub cell_data: GQLCells,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl From<DBCell> for Cell {
  fn from(db_cell: DBCell) -> Self {
    let cell_data = match db_cell
      .cell_type
      .as_str()
    {
      "BasicText" => {
        let c: BasicTextCell =
          serde_json::from_value(db_cell.cell_data).expect("Can't deserialize BasicTextCell");
        GQLCells::BasicTextCell(c)
      }
      "OwnerText" => {
        let c: OwnerTextCell =
          serde_json::from_value(db_cell.cell_data).expect("Can't deserialize OwnerTextCell");
        GQLCells::OwnerTextCell(c)
      }
      "GoogleSheets" => {
        let c: GoogleSheetsCell =
          serde_json::from_value(db_cell.cell_data).expect("Can't deserialize GoogleSheetsCell");
        GQLCells::GoogleSheetsCell(c)
      }
      &_ => GQLCells::Empty(EmptyCell {
        cell_type: String::from("nothing"),
      }),
    };

    let cell_type = CellTypes::from_str(
      db_cell
        .cell_type
        .as_str(),
    )
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

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewCell {
  pub portal_id: Uuid,

  pub dimensions: Vec<Uuid>,

  pub cell_type: CellTypes,

  pub cell_data: String,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct UpdateCell {
  pub id: Uuid,

  pub dimensions: Option<Vec<Uuid>>,

  // Going to require the cell_type for now so we know how to parse the cell_data if present.
  pub cell_type: CellTypes,

  #[graphql(description = "For now cell_data needs to be stringified, but it is type checked when parsed!")]
  pub cell_data: Option<String>,
  // pub cell_data: Option<serde_json::Value>,
}

impl Query {
  pub async fn cell_impl(ctx: &GQLContext, cell_id: Uuid) -> FieldResult<Cell> {
    get_cell(&ctx.pool, cell_id)
      .await
      .map(|db_cell| db_cell.into())
      .map_err(FieldError::from)
  }

  pub async fn cells_any_dimensions_impl(
    ctx: &GQLContext,
    dimension_ids: Vec<Uuid>,
  ) -> FieldResult<Vec<Cell>> {
    get_cells_with_any_dimensions(&ctx.pool, dimension_ids)
      .await
      .map(|db_cells| {
        db_cells
          .into_iter()
          .map(|db_cell| db_cell.into())
          .collect()
      })
      .map_err(FieldError::from)
  }

  pub async fn cells_all_dimensions_impl(
    ctx: &GQLContext,
    dimension_ids: Vec<Uuid>,
  ) -> FieldResult<Vec<Cell>> {
    get_cells_with_all_dimensions(&ctx.pool, dimension_ids)
      .await
      .map(|db_cells| {
        db_cells
          .into_iter()
          .map(|db_cell| db_cell.into())
          .collect()
      })
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_cell_impl(ctx: &GQLContext, new_cell: NewCell) -> FieldResult<Cell> {
    create_cell(&ctx.pool, &ctx.auth0_user_id, new_cell.into())
    .await
    .map(|db_cell| db_cell.into())
    .map_err(FieldError::from)
  }

  pub async fn update_cell_impl(ctx: &GQLContext, updated_cell: UpdateCell) -> FieldResult<Cell> {
    update_cell(&ctx.pool, &ctx.auth0_user_id, updated_cell.into())
      .await
      .map(|db_cell| db_cell.into())
      .map_err(FieldError::from)
  }
}
