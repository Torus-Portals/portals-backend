use juniper::{
  FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, GraphQLUnion,
};
use chrono::{DateTime, Utc};
use strum_macros::{Display, EnumString};
use uuid::Uuid;
use crate::graphql::context::GQLContext;

use super::sourcequeries::block_sourcequery::BlockSourceQuery;

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum SourceQueryTypes {
  Block
}

#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub enum GQLSourceQueries {
  Block(BlockSourceQuery),
}

pub struct SourceQuery {
  pub id: Uuid,

  pub sourcequery_type: SourceQueryTypes,

  pub sourcequery_data: GQLSourceQueries,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}