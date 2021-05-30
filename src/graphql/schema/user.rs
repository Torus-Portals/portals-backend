use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLObject};
use uuid::Uuid;

use super::Query;
use crate::graphql::context::GQLContext;
use crate::services::db::user_service::{DBUser};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct User {
  pub id: Uuid,

  pub auth0id: String,

  pub name: String,

  pub nickname: String,

  pub email: String,

  // TODO: Maybe try to figure out how to use postgres enums with status.
  pub status: String,

  pub orgs: Vec<Uuid>,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

impl From<DBUser> for User {
  fn from(db_user: DBUser) -> Self {
    User {
      id: db_user.id,
      auth0id: db_user.auth0id,
      name: db_user.name,
      nickname: db_user.nickname,
      email: db_user.email,
      status: db_user.status,
      orgs: db_user.orgs,
      created_at: db_user.created_at,
      created_by: db_user.created_by,
      updated_at: db_user.updated_at,
      updated_by: db_user.updated_by,
    }
  }
}

impl Query {
  pub async fn user_impl(ctx: &GQLContext, user_id: Uuid) -> FieldResult<User> {
    ctx
      .db
      .get_user(user_id)
      .await
      .map(|db_user| db_user.into())
      .map_err(FieldError::from)
  }
}
