use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::graphql::schema::{
  cell::{CellTypes, NewCell, UpdateCell},
  cells::{
    basic_text_cell::BasicTextCell, google_sheets_cell::GoogleSheetsCell,
    owner_text_cell::OwnerTextCell,
  },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DBCell {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  // NOTE: Should probably be good to make this an enum.
  #[serde(rename = "cellType")]
  pub cell_type: String,

  pub dimensions: Vec<Uuid>,

  pub cell_data: serde_json::Value,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBNewCell {
  pub portal_id: Uuid,

  pub dimensions: Vec<Uuid>,

  pub cell_type: String,

  pub cell_data: serde_json::Value,
}

impl From<NewCell> for DBNewCell {
  fn from(new_cell: NewCell) -> Self {
    let cell_data = cell_string_to_serde_value(&new_cell.cell_type, new_cell.cell_data)
      .expect("unable to convert cell data string to serde_json::Value");

    DBNewCell {
      portal_id: new_cell.portal_id,
      dimensions: new_cell.dimensions,
      cell_type: new_cell
        .cell_type
        .to_string(),
      cell_data,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBUpdateCell {
  pub id: Uuid,

  pub dimensions: Option<Vec<Uuid>>,

  pub cell_type: String,

  pub cell_data: Option<serde_json::Value>,
}

impl From<UpdateCell> for DBUpdateCell {
  fn from(update_cell: UpdateCell) -> Self {
    let cell_data = update_cell
      .cell_data
      .clone()
      .map(|cd| {
        cell_string_to_serde_value(&update_cell.cell_type, cd)
          .expect("unable to convert cell data string to serde_json::Value")
      });

    DBUpdateCell {
      id: update_cell.id,
      dimensions: update_cell.dimensions,
      cell_type: update_cell
        .cell_type
        .to_string(),
      cell_data,
    }
  }
}

pub async fn get_cell<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  cell_id: Uuid,
) -> Result<DBCell> {
  sqlx::query_as!(DBCell, "select * from cells where id = $1", cell_id)
    .fetch_one(pool)
    .await
    .map_err(anyhow::Error::from)
}

pub async fn create_cell<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
  new_cell: DBNewCell,
) -> Result<DBCell> {
  sqlx::query_as!(
    DBCell,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into cells (portal_id, dimensions, cell_type, cell_data, created_by, updated_by)
    values ($2, $3, $4, $5, (select id from _user), (select id from _user))
    returning *;
    "#,
    auth0_user_id,
    new_cell.portal_id,
    &new_cell.dimensions,
    new_cell.cell_type,
    new_cell.cell_data
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn update_cell<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
  update_cell: DBUpdateCell,
) -> Result<DBCell> {
  sqlx::query_as!(
    DBCell,
    r#"
    with _user as (select * from users where auth0id = $1)
    update cells
      set
        dimensions = coalesce($3, dimensions),
        cell_type = coalesce($4, cell_type),
        cell_data = coalesce($5, cell_data),
        updated_by = (select id from _user)
    where id = $2
    returning *;
    "#,
    auth0_user_id,
    update_cell.id,
    update_cell
      .dimensions
      .as_deref(),
    update_cell.cell_type,
    update_cell.cell_data
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

// Returns all cells that contain ANY of the dimensions that are passed to it.
pub async fn get_cells_with_any_dimensions<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  dimension_ids: Vec<Uuid>,
) -> Result<Vec<DBCell>> {
  sqlx::query_as!(
    DBCell,
    r#"select * from cells where $1::UUID[] && dimensions"#,
    &dimension_ids
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

// Returns all cells that contain EVERY dimension that is passed to it.
pub async fn get_cells_with_all_dimensions<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  dimension_ids: Vec<Uuid>,
) -> Result<Vec<DBCell>> {
  sqlx::query_as!(
    DBCell,
    r#"select * from cells where $1::UUID[] <@ dimensions"#,
    &dimension_ids
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub fn cell_string_to_serde_value(cell_type: &CellTypes, cd: String) -> Result<serde_json::Value> {
  let value = match cell_type {
    CellTypes::BasicText => {
      let cell: BasicTextCell = serde_json::from_str(&cd)?;
      serde_json::to_value(cell)
    }
    CellTypes::OwnerText => {
      let cell: OwnerTextCell = serde_json::from_str(&cd)?;
      serde_json::to_value(cell)
    }
    CellTypes::GoogleSheets => {
      let cell: GoogleSheetsCell = serde_json::from_str(&cd)?;
      serde_json::to_value(cell)
    }
  };

  value.map_err(anyhow::Error::from)
}
