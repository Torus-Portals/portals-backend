use crate::services::db::user_service::DBUser;
use chrono::{DateTime, Utc};
use juniper::{graphql_object, FieldError, FieldResult};
use uuid::Uuid;

use super::Query;
use crate::graphql::context::GQLContext;

use crate::graphql::schema::Org;

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
      org_ids: db_user.orgs,
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
      .map(|db_user| -> User { db_user.into() })
      .map_err(FieldError::from)

    // let user_orgs = db_user.orgs.clone();

    // let loaded_orgs = ctx
    //   .org_loader
    //   .load_many(db_user.orgs.into())
    //   .await;

    // let orgs = loaded_orgs
    //   .into_iter()
    //   .fold(Vec::new(), |mut acc, (id, oo)| {
    //     if user_orgs.contains(&id) {
    //       acc.push(oo)
    //     };
    //     acc
    //   });

    // Ok(User {
    //   id: db_user.id,
    //   auth0id: db_user.auth0id,
    //   name: db_user.name,
    //   nickname: db_user.nickname,
    //   email: db_user.email,
    //   status: db_user.status,
    //   org_ids: db_user.orgs,
    //   created_at: db_user.created_at,
    //   created_by: db_user.created_by,
    //   updated_at: db_user.updated_at,
    //   updated_by: db_user.updated_by,
    // })
  }

  pub async fn current_user_impl(ctx: &GQLContext) -> FieldResult<User> {
    ctx
      .db
      .get_user_by_auth0_id(&ctx.auth0_user_id)
      .await
      .map(|db_user| -> User { db_user.into() })
      .map_err(FieldError::from)

    // let user_orgs = db_user.orgs.clone();

    // let loaded_orgs = ctx
    //   .org_loader
    //   .load_many(db_user.orgs.into())
    //   .await;

    // let orgs = loaded_orgs
    //   .into_iter()
    //   .fold(Vec::new(), |mut acc, (id, oo)| {
    //     if user_orgs.contains(&id) {
    //       acc.push(oo)
    //     };
    //     acc
    //   });

    // Ok(User {
    //   id: db_user.id,
    //   auth0id: db_user.auth0id,
    //   name: db_user.name,
    //   nickname: db_user.nickname,
    //   email: db_user.email,
    //   status: db_user.status,
    //   org_ids: db_user.orgs,
    //   created_at: db_user.created_at,
    //   created_by: db_user.created_by,
    //   updated_at: db_user.updated_at,
    //   updated_by: db_user.updated_by,
    // })
  }
}
