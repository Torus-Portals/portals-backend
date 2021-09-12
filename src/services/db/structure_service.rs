use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::graphql::schema::structure::UpdateStructure;

#[derive(Debug, Serialize, Deserialize)]
pub struct DBStructure {
  pub id: Uuid,

  pub structure_type: String,

  pub structure_data: serde_json::Value,

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
pub struct DBNewStructure {
  pub structure_type: String,

  pub structure_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBUpdateStructure {
  pub id: Uuid,

  pub structure_type: Option<String>,

  pub structure_data: Option<serde_json::Value>,
}

impl From<UpdateStructure> for DBUpdateStructure {
  fn from(update_structure: UpdateStructure) -> Self {
    let structure_type = update_structure
      .structure_type
      .map(|st| st.to_string());
    let structure_data = serde_json::to_value(&update_structure.structure_data).ok();

    DBUpdateStructure {
      id: update_structure.id,
      structure_type,
      structure_data,
    }
  }
}

pub async fn get_structure<'e>(pool: impl Executor<'e, Database = Postgres>, structure_id: Uuid) -> Result<DBStructure> {
  sqlx::query_as!(
    DBStructure,
    "select * from structures where id = $1",
    structure_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_structures<'e>(pool: impl Executor<'e, Database = Postgres>, ids: &[Uuid]) -> Result<Vec<DBStructure>> {
  sqlx::query_as!(
    DBStructure,
    "select * from structures where id = any($1)",
    ids
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_structure<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
  new_structure: DBNewStructure,
) -> Result<DBStructure> {
  sqlx::query_as!(
    DBStructure,
    r#"
      with _user as (select * from users where auth0id = $1)
      insert into structures (
        structure_type,
        structure_data,
        created_by,
        updated_by
      ) values (
        $2,
        $3,
        (select id from _user),
        (select id from _user)
      ) returning *
      "#,
    auth0_user_id,
    new_structure.structure_type,
    new_structure.structure_data,
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn update_structure<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
  update_structure: DBUpdateStructure,
) -> Result<DBStructure> {
  sqlx::query_as!(
    DBStructure,
    r#"
      with _user as (select * from users where auth0id = $1)
      update structures
        set
          structure_type = coalesce($3, structure_type),
          structure_data = coalesce($4, structure_data)
      where id = $2
      returning *;
      "#,
    auth0_user_id,
    update_structure.id,
    update_structure.structure_type,
    update_structure.structure_data
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}
