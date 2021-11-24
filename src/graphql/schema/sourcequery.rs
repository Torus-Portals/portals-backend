use crate::graphql::context::GQLContext;
use crate::services::db::sourcequery_service::{
  create_sourcequery, get_sourcequery, DBSourceQuery,
};
use chrono::{DateTime, Utc};
use juniper::{
  FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, GraphQLUnion,
};
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::sourcequeries::table_block_sourcequery::TableBlockSourceQuery;
use super::Mutation;
use super::Query;

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum SourceQueryTypes {
  TableBlock,
}

#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub enum GQLSourceQueries {
  TableBlock(TableBlockSourceQuery),
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

impl From<DBSourceQuery> for SourceQuery {
  fn from(sourcequery: DBSourceQuery) -> Self {
    let sourcequery_data = match sourcequery.sourcequery_type.as_str() {
      "TableBlock" => {
        let b: TableBlockSourceQuery = serde_json::from_value(sourcequery.sourcequery_data)
          .expect("unable to deserialize blocksourcequery");
        GQLSourceQueries::TableBlock(b)
      }
      &_ => panic!("unknown sourcequery type"),
    };

    let sourcequery_type =
      SourceQueryTypes::from_str(sourcequery.sourcequery_type.as_str()).unwrap();

    SourceQuery {
      id: sourcequery.id,
      sourcequery_type,
      sourcequery_data,
      created_at: sourcequery.created_at,
      created_by: sourcequery.created_by,
      updated_at: sourcequery.updated_at,
      updated_by: sourcequery.updated_by,
    }
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
      .map(|db_sourcequery| db_sourcequery.into())
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_sourcequery_impl(
    ctx: &GQLContext,
    new_sourcequery: NewSourceQuery,
  ) -> FieldResult<SourceQuery> {
    create_sourcequery(&ctx.pool, &ctx.auth0_user_id, new_sourcequery.into())
      .await
      .map(|db_sourcequery| db_sourcequery.into())
      .map_err(FieldError::from)
  }
}
