use chrono::{DateTime, Utc};
use juniper::{
  FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, GraphQLUnion,
};
use strum_macros::{Display, EnumString};
use std::str::FromStr;
use uuid::Uuid;

use super::{
  sources::block_source::BlockSource, sources::empty_source::EmptySource, Mutation, Query,
};

use crate::graphql::context::GQLContext;
use crate::services::db::source_service::{DBSource, create_source, get_possible_sources};

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum SourceTypes {
  #[strum(serialize = "Block")]
  #[graphql(name = "Block")]
  Block,

  #[strum(serialize = "Empty")]
  #[graphql(name = "Empty")]
  Empty,
}

#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
pub enum GQLSources {
  Block(BlockSource),

  Empty(EmptySource),
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
  pub id: Uuid,

  pub source_type: SourceTypes,

  pub source_data: GQLSources,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl From<DBSource> for Source {
  fn from(db_source: DBSource) -> Self {
    let source_type = SourceTypes::from_str(
      db_source
        .source_type
        .as_str(),
    )
    .expect("Unable to convert source_type string to enum variant");
    
    let source_data = match source_type {
      SourceTypes::Block => GQLSources::Block(
        db_source
          .source_data
          .into(),
      ),
      SourceTypes::Empty => GQLSources::Empty(
        db_source
          .source_data
          .into(),
      ),
    };

    Source {
      id: db_source.id,
      source_type,
      source_data,
      created_at: db_source.created_at,
      created_by: db_source.created_by,
      updated_at: db_source.updated_at,
      updated_by: db_source.updated_by,
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSource {
  pub source_type: SourceTypes,

  pub source_data: String,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PossibleSource {
  pub source_type: SourceTypes,

  pub source_data: GQLSources,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct PossibleSourceInput {
  pub project_id: Uuid,

  pub dashboard_id: Uuid,

  pub page_id: Uuid,

  pub block_id: Uuid,
}

impl Query {
  pub async fn possible_sources_impl(
    ctx: &GQLContext,
    input: PossibleSourceInput,
  ) -> FieldResult<Vec<PossibleSource>> {
    let local_pool = ctx.pool.clone();
    let possible_sources = get_possible_sources(local_pool, input).await?;

    Ok(possible_sources)
  }
}

impl Mutation {
  pub async fn create_source_impl(ctx: &GQLContext, new_source: NewSource) -> FieldResult<Source> {
    create_source(&ctx.pool, &ctx.auth0_user_id, new_source.into())
      .await
      .map(|db_source| db_source.into())
      .map_err(FieldError::from)
  }
}
