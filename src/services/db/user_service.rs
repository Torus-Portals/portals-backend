use super::DB;

use anyhow::Result;
use chrono::{DateTime, Utc};

use uuid::Uuid;

use crate::graphql::schema::user::{NewUser, UpdateUser};

// DBUser

#[derive(Debug, Serialize)]
pub struct DBUser {
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

// DBNewUser
#[derive(Debug, Serialize)]
pub struct DBNewUser {
  pub name: String,

  pub nickname: String,

  pub email: String,

  pub status: String,

  pub org_ids: Vec<Uuid>,

  pub role_ids: Vec<Uuid>,
}

impl From<NewUser> for DBNewUser {
  fn from(new_user: NewUser) -> Self {
    DBNewUser {
      name: new_user.name,
      nickname: new_user.nickname,
      email: new_user.email,
      status: new_user.status,
      org_ids: new_user
        .org_ids
        .unwrap_or_else(|| vec![]),
      role_ids: new_user
        .role_ids
        .unwrap_or_else(|| vec![]),
    }
  }
}

#[derive(Debug, Serialize)]
pub struct DBUpdateUser {
  pub id: Uuid,

  pub name: Option<String>,

  pub nickname: Option<String>,

  pub email: Option<String>,

  pub status: Option<String>,

  pub org_ids: Option<Vec<Uuid>>,

  pub role_ids: Option<Vec<Uuid>>,
}

impl From<UpdateUser> for DBUpdateUser {
  fn from(update_user: UpdateUser) -> Self {
    DBUpdateUser {
      id: update_user.id,
      name: update_user.name,
      nickname: update_user.nickname,
      email: update_user.email,
      status: update_user.status,
      org_ids: update_user.org_ids,
      role_ids: update_user.role_ids,
    }
  }
}

impl DB {
  pub async fn user_exists(&self, auth0_user_id: &str) -> Result<bool> {
    sqlx::query!(
      "select exists(select 1 from users where auth0id = $1) as user_exists",
      auth0_user_id
    )
    .fetch_one(&self.pool)
    .await
    .map(|record| {
      record
        .user_exists
        .unwrap()
    })
    .map_err(anyhow::Error::from)
  }

  pub async fn get_user(&self, user_id: Uuid) -> Result<DBUser> {
    sqlx::query_as!(DBUser, "select * from users where id = $1", user_id)
      .fetch_one(&self.pool)
      .await
      .map_err(anyhow::Error::from)
  }

  pub async fn get_user_by_auth0_id(&self, auth0_user_id: &str) -> Result<DBUser> {
    sqlx::query_as!(
      DBUser,
      "select * from users where auth0id = $1",
      auth0_user_id
    )
    .fetch_one(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }

  pub async fn create_user(&self, auth0_user_id: &str, new_user: DBNewUser) -> Result<DBUser> {
    let system_uuid = Uuid::parse_str("11111111-2222-3333-4444-555555555555")?;
    sqlx::query_as!(
      DBUser,
      r#"
      insert into users (name, nickname, email, status, org_ids, role_ids, created_by, updated_by)
      values ($1, $2, $3, $4, $5, $6, $7, $7)
      returning *;
      "#,
      new_user.name,
      new_user.nickname,
      new_user.email,
      new_user.status,
      &new_user.org_ids,
      &new_user.role_ids,
      system_uuid
    )
    .fetch_one(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }

  // Might be a good optimization for the future to use something like:
  // "param_1 IS NOT NULL AND param_1 IS DISTINCT FROM column_1" found in this question:
  // https://stackoverflow.com/questions/13305878/dont-update-column-if-update-value-is-null
  pub async fn update_user(
    &self,
    auth0_user_id: &str,
    update_user: DBUpdateUser,
  ) -> Result<DBUser> {
    sqlx::query_as!(
      DBUser,
      r#"
      with _user as (select * from users where auth0id = $1)
      update users
        set
          name = coalesce($3, name),
          nickname = coalesce($4, nickname),
          email = coalesce($5, email),
          org_ids = coalesce($6, org_ids),
          role_ids = coalesce($7, role_ids),
          status = coalesce($8, status),
          updated_by = (select id from _user)
      where id = $2
      returning *;
      "#,
      auth0_user_id,
      update_user.id,
      update_user.name,
      update_user.nickname,
      update_user.email,
      update_user
        .org_ids
        .as_deref(),
      update_user
        .role_ids
        .as_deref(),
      update_user.status
    )
    .fetch_one(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }
}
