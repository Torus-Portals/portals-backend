use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::graphql::schema::{
  sourcequeries::{
    cells_block_sourcequery::CellsBlockSourceQuery, table_block_sourcequery::TableBlockSourceQuery,
    text_block_sourcequery::TextBlockSourceQuery,
    xy_chart_block_sourcequery::XYChartBlockSourceQuery,
  },
  sourcequery::{NewSourceQuery, SourceQueryTypes},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DBSourceQuery {
  pub id: Uuid,

  pub sourcequery_type: String,

  pub sourcequery_data: serde_json::Value,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

pub struct DBNewSourceQuery {
  pub sourcequery_type: String,

  pub sourcequery_data: serde_json::Value,
}

impl From<NewSourceQuery> for DBNewSourceQuery {
  fn from(new_sourcequery: NewSourceQuery) -> Self {
    let sourcequery_data = sq_string_to_serde_value(
      &new_sourcequery.sourcequery_type,
      new_sourcequery.sourcequery_data,
    )
    .expect("Unable to convert sourcequery data string to serde_json::Value");

    Self {
      sourcequery_type: new_sourcequery
        .sourcequery_type
        .to_string(),
      sourcequery_data,
    }
  }
}

pub async fn get_sourcequery(
  pool: impl PgExecutor<'_>,
  sourcequery_id: Uuid,
) -> Result<DBSourceQuery> {
  sqlx::query_as!(
    DBSourceQuery,
    r#"
    select
      id,
      sourcequery_type,
      sourcequery_data,
      created_at,
      created_by,
      updated_at,
      updated_by
    from
      sourcequeries
    where
      id = $1
    "#,
    sourcequery_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_sourcequeries(
  pool: impl PgExecutor<'_>,
  sourcequery_ids: Vec<Uuid>,
) -> Result<Vec<DBSourceQuery>> {
  sqlx::query_as!(
    DBSourceQuery,
    r#"
    select
      id,
      sourcequery_type,
      sourcequery_data,
      created_at,
      created_by,
      updated_at,
      updated_by
    from
      sourcequeries
    where
      id = any($1)
    "#,
    &sourcequery_ids
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_sourcequery(
  pool: impl PgExecutor<'_>,
  auth0_user_id: &str,
  new_sourcequery: DBNewSourceQuery,
) -> Result<DBSourceQuery> {
  sqlx::query_as!(
    DBSourceQuery,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into sourcequeries (
      sourcequery_type,
      sourcequery_data,
      created_by,
      updated_by
    )
    values (
      $2,
      $3,
      (select id from _user),
      (select id from _user)
    )
    returning *;
    "#,
    auth0_user_id,
    new_sourcequery.sourcequery_type,
    new_sourcequery.sourcequery_data,
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub fn sq_string_to_serde_value(
  sq_type: &SourceQueryTypes,
  sq_data: String,
) -> Result<serde_json::Value> {
  let value = match sq_type {
    SourceQueryTypes::TableBlock => {
      let bsq: TableBlockSourceQuery = serde_json::from_str(&sq_data)?;
      serde_json::to_value(bsq)
    }
    SourceQueryTypes::TextBlock => {
      let tsq: TextBlockSourceQuery = serde_json::from_str(&sq_data)?;
      serde_json::to_value(tsq)
    }
    SourceQueryTypes::CellsBlock => {
      let csq: CellsBlockSourceQuery = serde_json::from_str(&sq_data)?;
      serde_json::to_value(csq)
    }
    SourceQueryTypes::XYChartBlock => {
      let xysq: XYChartBlockSourceQuery = serde_json::from_str(&sq_data)?;
      serde_json::to_value(xysq)
    }
  };

  value.map_err(anyhow::Error::from)
}
