use crate::services::db::user_service::DBUser;
use chrono::{DateTime, Utc};
use juniper::{graphql_object, FieldError, FieldResult, GraphQLInputObject};
use uuid::Uuid;

use super::Mutation;
use super::Query;
use crate::graphql::context::GQLContext;

use crate::graphql::schema::Org;

// User

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
  pub id: Uuid,

  pub auth0id: String,

  pub name: String,

  pub nickname: String,

  pub email: String,

  // TODO: Maybe try to figure out how to use postgres enums with status.
  pub status: String,

  pub org_ids: Vec<Uuid>,

  pub role_ids: Vec<Uuid>,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[graphql_object(context = GQLContext)]
impl User {
  fn id(&self) -> Uuid {
    self.id
  }

  fn auth0id(&self) -> String {
    self.auth0id.clone()
  }

  fn name(&self) -> String {
    self.name.clone()
  }

  fn nickname(&self) -> String {
    self
      .nickname
      .clone()
  }

  fn email(&self) -> String {
    self.email.clone()
  }

  fn status(&self) -> String {
    self.status.clone()
  }

  fn org_ids(&self) -> Vec<Uuid> {
    self.org_ids.clone()
  }

  pub async fn orgs(&self, context: &GQLContext) -> Vec<Org> {
    let org_map = context
      .org_loader
      .load_many(self.org_ids.clone())
      .await;

    let orgs = org_map
      .into_iter()
      .fold(vec![], |mut acc, (_, org)| {
        acc.push(org);
        acc
      });

    orgs
  }

  fn role_ids(&self) -> Vec<Uuid> {
    self
      .role_ids
      .clone()
  }

  fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  fn created_by(&self) -> Uuid {
    self.created_by
  }

  fn updated_at(&self) -> DateTime<Utc> {
    self.updated_at
  }

  fn updated_by(&self) -> Uuid {
    self.updated_by
  }
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
      org_ids: db_user.org_ids,
      role_ids: db_user.role_ids,
      created_at: db_user.created_at,
      created_by: db_user.created_by,
      updated_at: db_user.updated_at,
      updated_by: db_user.updated_by,
    }
  }
}

// NewUser

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewUser {
  pub name: String,

  pub nickname: String,

  pub email: String,

  pub status: String,

  pub org_ids: Option<Vec<Uuid>>,

  pub role_ids: Option<Vec<Uuid>>,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct UpdateUser {
  pub id: Uuid,

  pub name: Option<String>,

  pub nickname: Option<String>,

  pub email: Option<String>,

  pub status: Option<String>,

  pub org_ids: Option<Vec<Uuid>>,

  pub role_ids: Option<Vec<Uuid>>,
}

impl Query {
  pub async fn user_impl(ctx: &GQLContext, user_id: Uuid) -> FieldResult<User> {
    ctx
      .db
      .get_user(user_id)
      .await
      .map(|db_user| -> User { db_user.into() })
      .map_err(FieldError::from)
  }

  pub async fn current_user_impl(ctx: &GQLContext) -> FieldResult<User> {
    let user_exists = ctx
      .db
      .user_exists(&ctx.auth0_user_id)
      .await?;

    if user_exists {
      ctx
        .db
        .get_user_by_auth0_id(&ctx.auth0_user_id)
        .await
        .map(|db_user| -> User { db_user.into() })
        .map_err(FieldError::from)
    } else {
      let mut auth_api = ctx
        .auth0_api
        .lock()
        .await;

      let auth0user = auth_api
        .get_auth0_user(&ctx.auth0_user_id)
        .await?;

      let new_user = NewUser {
        name: auth0user.name,
        nickname: auth0user.nickname,
        email: auth0user.email,
        status: String::from("active"),
        org_ids: None,
        role_ids: None,
      };

      let db_user = ctx
        .db
        .create_user(&ctx.auth0_user_id, new_user.into())
        .await?;

      Ok(db_user.into())
    }
  }
}

impl Mutation {
  // TODO: Need to figure out if it will be needed for "users" to be the creator of other users, 
  //       of if it will always be the "system".
  pub async fn create_user_impl(ctx: &GQLContext, new_user: NewUser) -> FieldResult<User> {
    ctx
      .db
      .create_user(&ctx.auth0_user_id, new_user.into())
      .await
      .map(|db_user| -> User { db_user.into() })
      .map_err(FieldError::from)
  }

  pub async fn update_user_impl(ctx: &GQLContext, update_user: UpdateUser) -> FieldResult<User> {
    ctx
      .db
      .update_user(&ctx.auth0_user_id, update_user.into())
      .await
      .map(|db_user| -> User { db_user.into() })
      .map_err(FieldError::from)
  }
}
