use crate::services::db::user_service::user_exists_by_email;
use crate::services::db::user_service::{
  create_user, create_user_with_new_org, get_user, get_user_by_auth0_id, get_user_by_email, get_users, update_user,
  auth0_user_exists, DBNewUser, DBUser,
};
use chrono::{DateTime, Utc};
use juniper::{graphql_object, FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use uuid::Uuid;

use super::Mutation;
use super::Query;
use crate::graphql::context::GQLContext;

use crate::graphql::schema::Org;

// User

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
  pub id: Uuid,

  pub auth0id: String,

  pub name: String,

  pub nickname: String,

  pub email: String,

  // TODO: Maybe try to figure out how to use postgres enums with status.
  pub user_status: UserStatus,

  pub org_ids: Vec<Uuid>,

  pub role_ids: Vec<Uuid>,

  pub project_ids: Vec<Uuid>,

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

  fn user_status(&self) -> UserStatus {
    self.user_status.clone()
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

  fn project_ids(&self) -> Vec<Uuid> {
    self.project_ids.clone()
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
      // status: db_user.status,
      user_status: serde_json::from_value(db_user.user_status).expect("Unable to deserialize UserStatus"),
      org_ids: db_user.org_ids,
      role_ids: db_user.role_ids,
      project_ids: db_user.project_ids,
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

  pub user_status: UserStatusInput,

  pub org_ids: Option<Vec<Uuid>>,

  pub role_ids: Option<Vec<Uuid>>,
}

impl NewUser {
  pub fn into_db_new_user(&self, auth0id: String) -> DBNewUser {
    // I'm sure there is a better way to do this besides all these clones...
    DBNewUser {
      auth0id,
      name: self.name.clone(),
      nickname: self
        .nickname
        .clone(),
      email: self.email.clone(),
      user_status: serde_json::to_value(&self.user_status).expect("Unable to serialize UserStatusInput"),
      org_ids: self
        .org_ids
        .clone()
        .unwrap_or_else(|| vec![]),
      role_ids: self
        .role_ids
        .clone()
        .unwrap_or_else(|| vec![]),
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct UpdateUser {
  pub id: Uuid,

  pub auth0id: Option<String>,

  pub name: Option<String>,

  pub nickname: Option<String>,

  pub email: Option<String>,

  pub user_status: Option<UserStatusInput>,

  pub org_ids: Option<Vec<Uuid>>,

  pub role_ids: Option<Vec<Uuid>>,
}

// Not sure what's the best way to avoid duplication to satisfy GQL here
#[derive(GraphQLObject, Clone, Debug, Serialize, Deserialize)]
pub struct UserStatus {
  pub active: bool,

  pub has_logged_in: bool,

  pub password_changed: bool,

  pub onboarding_complete: bool,
}

#[derive(GraphQLInputObject, Clone, Debug, Serialize, Deserialize)]
pub struct UserStatusInput {
  pub active: bool,

  pub has_logged_in: bool,

  pub password_changed: bool,

  pub onboarding_complete: bool,
}

impl Default for UserStatusInput {
  fn default() -> Self {
    UserStatusInput {
      active: true,
      has_logged_in: false,
      password_changed: false,
      onboarding_complete: false,
    }
  }
}

impl Query {
  pub async fn user_impl(ctx: &GQLContext, user_id: Uuid) -> FieldResult<User> {
    get_user(&ctx.pool, user_id)
      .await
      .map(|db_user| -> User { db_user.into() })
      .map_err(FieldError::from)
  }
  
  pub async fn user_by_email_impl(ctx: &GQLContext, user_email: String) -> FieldResult<User> {
    get_user_by_email(&ctx.pool, &user_email)
      .await
      .map(|db_user| -> User { db_user.into() })
      .map_err(FieldError::from)
  }

  pub async fn current_user_impl(ctx: &GQLContext) -> FieldResult<User> {
    let user_exists = auth0_user_exists(&ctx.pool, &ctx.auth0_user_id).await?;

    if user_exists {
      get_user_by_auth0_id(&ctx.pool, &ctx.auth0_user_id)
        .await
        .map(|db_user| -> User { db_user.into() })
        .map_err(FieldError::from)
    } else {
      println!("user does not exist");
      let mut auth_api = ctx
        .auth0_api
        .lock()
        .await;

      let auth0user = auth_api
        .get_auth0_user(&ctx.auth0_user_id)
        .await?;

      // TODO: Need to come up with a check for auth0user.email_verified. 
      //       Might need to still query the Auth0 service until it's done.

      dbg!(&auth0user);

      let email_user_exists = user_exists_by_email(&ctx.pool, &auth0user.email).await?;
      if email_user_exists {
        // update user
        println!("email user exists!");
        let user = get_user_by_email(&ctx.pool, &auth0user.email).await?;

        let user_update = UpdateUser {
            id: user.id,
            auth0id: Some(auth0user.user_id),
            name: Some(auth0user.name),
            nickname: Some(auth0user.nickname),
            email: None,
            // status: Some(String::from("active")),
            user_status: Some(UserStatusInput::default()),
            org_ids: None,
            role_ids: None,
        };

        let updated_user = update_user(&ctx.pool, &ctx.auth0_user_id, user_update.into()).await?;

        Ok(updated_user.into())
      } else {
        println!("email user does not exist!");
        // create new user (and org for now)
        let new_user = NewUser {
          name: auth0user.name,
          nickname: auth0user.nickname,
          email: auth0user.email,
          user_status: UserStatusInput::default(),
          org_ids: None,
          role_ids: None,
        };
  
        // While we figure out what to do with Orgs, each user will have a personal org created when they are first created.
        // This will help in making sure that portals are coupled to Orgs, and not users.
        let user_and_org = create_user_with_new_org(
          ctx.pool.clone(),
          &ctx.auth0_user_id,
          new_user.into_db_new_user(
            ctx
              .auth0_user_id
              .to_owned(),
          ),
        )
        .await?;
  
        Ok(
          user_and_org
            .0
            .into(),
        )
      }
    }
  }

  pub async fn users_impl(ctx: &GQLContext, user_ids: Vec<Uuid>) -> FieldResult<Vec<User>> {
    get_users(&ctx.pool, user_ids)
      .await
      .map(|db_users| {
        db_users
          .into_iter()
          .map(|db_user| db_user.into())
          .collect()
      })
      .map_err(FieldError::from)
  }
}

impl Mutation {
  // TODO: Need to figure out if it will be needed for "users" to be the creator of other users,
  //       of if it will always be the "system".
  pub async fn create_user_impl(ctx: &GQLContext, new_user: NewUser) -> FieldResult<User> {
    create_user(
      &ctx.pool,
      new_user.into_db_new_user(
        ctx
          .auth0_user_id
          .to_owned(),
      ),
    )
    .await
    .map(|db_user| -> User { db_user.into() })
    .map_err(FieldError::from)
  }

  pub async fn update_user_impl(ctx: &GQLContext, updated_user: UpdateUser) -> FieldResult<User> {
    update_user(&ctx.pool, &ctx.auth0_user_id, updated_user.into())
      .await
      .map(|db_user| -> User { db_user.into() })
      .map_err(FieldError::from)
  }
}
