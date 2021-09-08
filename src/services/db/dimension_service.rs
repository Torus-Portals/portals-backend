use crate::graphql::schema::dimension::NewDimension;

use anyhow::Result;
use chrono::{DateTime, Utc};

use sqlx::{Executor, Postgres};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct DBDimension {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  #[serde(rename = "dimensionType")]
  pub dimension_type: String,

  pub meta: serde_json::Value,

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
}

impl From<NewDimension> for DBNewDimension {
  fn from(new_dim: NewDimension) -> Self {
    DBNewDimension {
      portal_id: new_dim.portal_id,
      name: new_dim.name,
      dimension_type: new_dim
        .dimension_type
        .to_string(),
    }
  }
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

pub async fn create_dimensions<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
  new_dimensions: Vec<DBNewDimension>,
) -> Result<Vec<DBDimension>> {
  let portal_ids = new_dimensions
    .iter()
    .map(|nd| nd.portal_id.clone())
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

  sqlx::query_as!(
    DBDimension,
    r#"
      with _user as (select * from users where auth0id = $1)
      insert into dimensions (
        portal_id,
        name,
        dimension_type,
        created_by,
        updated_by
      ) select * from unnest(
        $3::UUID[], 
        $4::TEXT[], 
        $5::TEXT[], 
        array_fill((select id from _user), ARRAY[$2::INT]::INT[]),
        array_fill((select id from _user), ARRAY[$2::INT]::INT[])
      )
      returning *;
      "#,
    auth0_user_id,
    (new_dimensions.len() as i32),
    &portal_ids,
    &names,
    &dimension_types
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn delete_portal_dimensions<'e>(pool: impl Executor<'e, Database = Postgres>, portal_id: Uuid) -> Result<i32> {
  sqlx::query!("delete from dimensions where portal_id = $1", portal_id)
  .execute(pool)
  .await
  .map(|qr| qr.rows_affected() as i32)
  .map_err(anyhow::Error::from)
}
