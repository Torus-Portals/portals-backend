use crate::graphql::schema::{
  dimension::{DimensionTypes, NewDimension},
  dimensions::{
    basic_table_column_dimension::BasicTableColumnDimension,
    basic_table_row_dimension::BasicTableRowDimension, empty_dimension::EmptyDimension,
    google_sheets_column_dimension::GoogleSheetsColumnDimension,
    google_sheets_row_dimension::GoogleSheetsRowDimension,
    owner_text_dimension::OwnerTextDimension, portal_member_dimension::PortalMemberDimension,
  },
};

use anyhow::Result;
use chrono::{DateTime, Utc};

use sqlx::{Executor, PgExecutor, Postgres};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct DBDimension {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  #[serde(rename = "dimensionType")]
  pub dimension_type: String,

  pub dimension_data: serde_json::Value,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[derive(Debug, Clone)]
pub struct DBNewDimension {
  pub portal_id: Uuid,

  pub name: String,

  pub dimension_type: String,

  pub dimension_data: serde_json::Value,
}

impl From<NewDimension> for DBNewDimension {
  fn from(new_dim: NewDimension) -> Self {
    let dimension_data = match &new_dim.dimension_type {
      DimensionTypes::BasicTableRow => {
        let dim: BasicTableRowDimension = serde_json::from_str(&new_dim.dimension_data)
          .expect("Unable to parse BasicTableRowDimension data");
        serde_json::to_value(dim)
          .expect("Unable to convert BasicTableRowDimension back to serde_json::Value")
      }
      DimensionTypes::BasicTableColumn => {
        let dim: BasicTableColumnDimension = serde_json::from_str(&new_dim.dimension_data)
          .expect("Unable to parse BasicTableColumnDimension data");
        serde_json::to_value(dim)
          .expect("Unable to convert BasicTableColumnDimension back to serde_json::Value")
      }
      DimensionTypes::PortalMember => {
        let dim: PortalMemberDimension = serde_json::from_str(&new_dim.dimension_data)
          .expect("Unable to parse PortalMemberDimension data");
        serde_json::to_value(dim)
          .expect("Unable to convert PortalMemberDimension back to serde_json::Value")
      }
      DimensionTypes::OwnerText => {
        let dim: OwnerTextDimension = serde_json::from_str(&new_dim.dimension_data)
          .expect("Unable to parse OwnerTextDimension data");
        serde_json::to_value(dim)
          .expect("Unable to convert OwnerTextDimension back to serde_json::Value")
      }
      DimensionTypes::GoogleSheetsRow => {
        let dim: GoogleSheetsRowDimension = serde_json::from_str(&new_dim.dimension_data)
          .expect("Unable to parse GoogleSheetsRow data");
        serde_json::to_value(dim)
          .expect("Unable to convert GoogleSheetsRow back to serde_json::Value")
      }
      DimensionTypes::GoogleSheetsColumn => {
        let dim: GoogleSheetsColumnDimension = serde_json::from_str(&new_dim.dimension_data)
          .expect("Unable to parse GoogleSheetsColumn data");
        serde_json::to_value(dim)
          .expect("Unable to convert GoogleSheetsColumn back to serde_json::Value")
      }
      DimensionTypes::Empty => {
        let dim: EmptyDimension = serde_json::from_str(&new_dim.dimension_data)
          .expect("Unable to parse EmptyDimension data");
        serde_json::to_value(dim)
          .expect("Unable to convert EmptyDimension back to serde_json::Value")
      }
    };

    DBNewDimension {
      portal_id: new_dim.portal_id,
      name: new_dim.name,
      dimension_type: new_dim
        .dimension_type
        .to_string(),
      dimension_data,
    }
  }
}

pub async fn get_dimension<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  dimension_id: Uuid,
) -> Result<DBDimension> {
  sqlx::query_as!(
    DBDimension,
    r#"select * from dimensions where id = $1 "#,
    dimension_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_dimensions<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  portal_id: Uuid,
) -> Result<Vec<DBDimension>> {
  sqlx::query_as!(
    DBDimension,
    r#"select * from dimensions where portal_id = $1 "#,
    portal_id
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_dimension<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
  new_dim: DBNewDimension,
) -> Result<DBDimension> {
  sqlx::query_as!(
    DBDimension,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into dimensions (portal_id, name, dimension_type, dimension_data, created_by, updated_by)
    values ($2, $3, $4, $5, (select id from _user), (select id from _user))
    returning *;
    "#,
    auth0_user_id,
    new_dim.portal_id,
    new_dim.name,
    new_dim.dimension_type,
    new_dim.dimension_data
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_dimensions<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
  new_dimensions: Vec<DBNewDimension>,
) -> Result<Vec<DBDimension>> {
  let portal_ids = new_dimensions
    .iter()
    .map(|nd| nd.portal_id)
    .collect::<Vec<Uuid>>();

  let names = new_dimensions
    .iter()
    .map(|nd| nd.name.clone())
    .collect::<Vec<String>>();

  let dimension_types = new_dimensions
    .iter()
    .map(|nd| {
      nd.dimension_type
        .clone()
    })
    .collect::<Vec<String>>();

  let dimension_datas = new_dimensions
    .iter()
    .map(|nd| {
      nd.dimension_data
        .clone()
    })
    .collect::<Vec<serde_json::Value>>();

  sqlx::query_as!(
    DBDimension,
    r#"
      with _user as (select * from users where auth0id = $1)
      insert into dimensions (
        portal_id,
        name,
        dimension_type,
        dimension_data,
        created_by,
        updated_by
      ) select * from unnest(
        $3::UUID[],
        $4::TEXT[],
        $5::TEXT[],
        $6::JSONB[],
        array_fill((select id from _user), ARRAY[$2::INT]::INT[]),
        array_fill((select id from _user), ARRAY[$2::INT]::INT[])
      )
      returning *;
      "#,
    auth0_user_id,
    (new_dimensions.len() as i32),
    &portal_ids,
    &names,
    &dimension_types,
    &dimension_datas,
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn delete_portal_dimensions(pool: impl PgExecutor<'_>, portal_id: Uuid) -> Result<i32> {
  sqlx::query!("delete from dimensions where portal_id = $1", portal_id)
    .execute(pool)
    .await
    .map(|qr| qr.rows_affected() as i32)
    .map_err(anyhow::Error::from)
}
