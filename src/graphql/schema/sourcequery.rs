use crate::graphql::context::GQLContext;
use crate::services::db::sourcequery_service::{
  create_sourcequery, get_sourcequery, DBSourceQuery,
};
use chrono::{DateTime, Utc};
use juniper::{
  FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, GraphQLUnion,
};
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::Mutation;
use super::Query;

use super::sourcequeries::table_block_sourcequery::TableBlockSourceQuery;
use super::sourcequeries::text_block_sourcequery::TextBlockSourceQuery;
use super::sourcequeries::cells_block_sourcequery::CellsBlockSourceQuery;
use super::sourcequeries::xy_chart_block_sourcequery::XYChartBlockSourceQuery;

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum SourceQueryTypes {
  #[strum(serialize = "TableBlock")]
  #[graphql(name = "TableBlock")]
  TableBlock,

  #[strum(serialize = "TextBlock")]
  #[graphql(name = "TextBlock")]
  TextBlock,

  #[strum(serialize = "CellsBlock")]
  #[graphql(name = "CellsBlock")]
  CellsBlock,

  #[strum(serialize = "XYChartBlock")]
  #[graphql(name = "XYChartBlock")]
  XYChartBlock,
}

#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub enum GQLSourceQueries {
  TableBlock(TableBlockSourceQuery),
  TextBlock(TextBlockSourceQuery),
  CellsBlock(CellsBlockSourceQuery),
  XYChartBlock(XYChartBlockSourceQuery),
}

impl GQLSourceQueries {
  pub fn from_serde_value(
    value: serde_json::Value,
    sq_type: &SourceQueryTypes,
  ) -> anyhow::Result<Self> {
    match sq_type {
      SourceQueryTypes::TableBlock => {
        let table_block_sourcequery: TableBlockSourceQuery = serde_json::from_value(value)?;
        Ok(GQLSourceQueries::TableBlock(table_block_sourcequery))
      }
      SourceQueryTypes::TextBlock => {
        let text_block_sourcequery: TextBlockSourceQuery = serde_json::from_value(value)?;
        Ok(GQLSourceQueries::TextBlock(text_block_sourcequery))
      }
      SourceQueryTypes::CellsBlock => {
        let cells_block_sourcequery: CellsBlockSourceQuery = serde_json::from_value(value)?;
        Ok(GQLSourceQueries::CellsBlock(cells_block_sourcequery))
      }
      SourceQueryTypes::XYChartBlock => {
        let xy_chart_block_sourcequery: XYChartBlockSourceQuery = serde_json::from_value(value)?;
        Ok(GQLSourceQueries::XYChartBlock(xy_chart_block_sourcequery))
      }
    }
  }
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
#[serde(rename_all = "camelCase")]
pub struct SourceQuery {
  pub id: Uuid,

  pub sourcequery_type: SourceQueryTypes,

  pub sourcequery_data: GQLSourceQueries,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl TryFrom<DBSourceQuery> for SourceQuery {
  type Error = anyhow::Error;

  fn try_from(sourcequery: DBSourceQuery) -> anyhow::Result<Self> {
    let sourcequery_type = SourceQueryTypes::from_str(
      sourcequery
        .sourcequery_type
        .as_str(),
    )?;

    let sourcequery_data =
      GQLSourceQueries::from_serde_value(sourcequery.sourcequery_data, &sourcequery_type)?;

    Ok(SourceQuery {
      id: sourcequery.id,
      sourcequery_type,
      sourcequery_data,
      created_at: sourcequery.created_at,
      created_by: sourcequery.created_by,
      updated_at: sourcequery.updated_at,
      updated_by: sourcequery.updated_by,
    })
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSourceQuery {
  pub sourcequery_type: SourceQueryTypes,

  pub sourcequery_data: String,
}

impl Query {
  pub async fn sourcequery_impl(
    ctx: &GQLContext,
    sourcequery_id: Uuid,
  ) -> FieldResult<SourceQuery> {
    get_sourcequery(&ctx.pool, sourcequery_id)
      .await
      .map(|db_sourcequery| {
        db_sourcequery
          .try_into()
          .map_err(FieldError::from)
      })
      .map_err(FieldError::from)?
  }
}

impl Mutation {
  pub async fn create_sourcequery_impl(
    ctx: &GQLContext,
    new_sourcequery: NewSourceQuery,
  ) -> FieldResult<SourceQuery> {
    create_sourcequery(&ctx.pool, &ctx.auth0_user_id, new_sourcequery.into())
      .await
      .map(|db_sourcequery| {
        db_sourcequery
          .try_into()
          .map_err(FieldError::from)
      })
      .map_err(FieldError::from)?
  }
}
