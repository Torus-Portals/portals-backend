use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use super::block_service::get_project_blocks;
use crate::graphql::schema::{
  source::{GQLSources, NewSource, PossibleSource, PossibleSourceInput, SourceTypes},
  sources::{block_source::BlockSource, empty_source::EmptySource},
};

#[derive(Debug, Clone)]
pub struct DBSource {
  pub id: Uuid,

  pub source_type: String,

  pub source_data: serde_json::Value,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,

  pub deleted_at: Option<DateTime<Utc>>,

  pub deleted_by: Option<Uuid>,
}

pub struct DBNewSource {
  pub source_type: String,

  pub source_data: serde_json::Value,
}

impl From<NewSource> for DBNewSource {
  fn from(new_source: NewSource) -> Self {
    let source_data = source_string_to_serde_value(&new_source.source_type, new_source.source_data)
      .expect("Failed to convert source data to serde value");

    Self {
      source_type: new_source.source_type.to_string(),
      source_data,
    }
  }
}

pub async fn get_possible_sources(
  pool: PgPool,
  input: PossibleSourceInput,
) -> Result<Vec<PossibleSource>> {
  let mut tx = pool.begin().await?;

  let project_blocks = get_project_blocks(&mut tx, input.project_id).await?;

  let possible_sources = project_blocks
    .into_iter()
    .filter(|block| block.id != input.block_id)
    .map(|block| {
      let block_source = BlockSource { block_id: block.id };

      PossibleSource {
        source_type: SourceTypes::Block,
        source_data: GQLSources::Block(block_source),
      }
    })
    .collect::<Vec<PossibleSource>>();

  tx.commit().await?;

  Ok(possible_sources)
}

pub async fn create_source(
  pool: impl PgExecutor<'_>,
  auth0_user_id: &str,
  new_source: DBNewSource,
) -> Result<DBSource> {
  sqlx::query_as!(
    DBSource,
    r#"
    with _user as (select id from users where auth0id = $1)
    insert into sources (source_type, source_data, created_by, updated_by)
    values ($2, $3, (select id from _user), (select id from _user))
    returning *;
    "#,
    auth0_user_id,
    new_source.source_type,
    new_source.source_data
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub fn source_string_to_serde_value(
  source_type: &SourceTypes,
  sd: String,
) -> Result<serde_json::Value> {
  let value = match source_type {
    SourceTypes::Block => {
      let block_source: BlockSource = serde_json::from_str(&sd)?;
      serde_json::to_value(block_source)
    }
    SourceTypes::Empty => {
      let empty_source: EmptySource = serde_json::from_str(&sd)?;
      serde_json::to_value(empty_source)
    }
  };

  value.map_err(anyhow::Error::from)
}
