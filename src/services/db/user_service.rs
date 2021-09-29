use anyhow::Result;
use chrono::{DateTime, Utc};

use sqlx::{Executor, Postgres, PgPool};
use uuid::Uuid;

use crate::graphql::schema::user::UpdateUser;

use super::org_service::{DBNewOrg, DBOrg, create_org};

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
#[derive(Debug, Clone, Serialize)]
pub struct DBNewUser {
  pub auth0id: String,

  pub name: String,

  pub nickname: String,

  pub email: String,

  pub status: String,

  pub org_ids: Vec<Uuid>,

  pub role_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct DBUpdateUser {
  pub id: Uuid,

  pub auth0id: Option<String>,

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
      auth0id: update_user.auth0id,
      name: update_user.name,
      nickname: update_user.nickname,
      email: update_user.email,
      status: update_user.status,
      org_ids: update_user.org_ids,
      role_ids: update_user.role_ids,
    }
  }
}

pub async fn auth0_user_exists<'e>(pool: impl Executor<'e, Database = Postgres>, auth0_user_id: &str) -> Result<bool> {
  sqlx::query!(
    "select exists(select 1 from users where auth0id = $1) as user_exists",
    auth0_user_id
  )
  .fetch_one(pool)
  .await
  .map(|record| {
    record
      .user_exists
      .unwrap()
  })
  .map_err(anyhow::Error::from)
}

pub async fn _user_exists<'e>(pool: impl Executor<'e, Database = Postgres>, user_id: Uuid) -> Result<bool> {
  sqlx::query!(
    "select exists(select 1 from users where id = $1) as user_exists",
    user_id
  )
  .fetch_one(pool)
  .await
  .map(|record| {
    record
      .user_exists
      .unwrap()
  })
  .map_err(anyhow::Error::from)
}

pub async fn user_exists_by_email<'e>(pool: impl Executor<'e, Database = Postgres>, email: &str) -> Result<bool> {
  sqlx::query!(
    "select exists(select 1 from users where email = $1) as user_exists",
    email
  )
  .fetch_one(pool)
  .await
  .map(|record| {
    record
      .user_exists
      .unwrap()
  })
  .map_err(anyhow::Error::from)
}

pub async fn get_user<'e>(pool: impl Executor<'e, Database = Postgres>, user_id: Uuid) -> Result<DBUser> {
  sqlx::query_as!(DBUser, "select * from users where id = $1", user_id)
    .fetch_one(pool)
    .await
    .map_err(anyhow::Error::from)
}

pub async fn get_user_by_auth0_id<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
) -> Result<DBUser> {
  sqlx::query_as!(
    DBUser,
    "select * from users where auth0id = $1;",
    auth0_user_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_user_by_email<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  email: &str,
) -> Result<DBUser> {
  sqlx::query_as!(
    DBUser,
    "select * from users where email = $1;",
    email
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

// NOTE: This is not at all secure, and some kinda permissions check for this should be done in the future. 
pub async fn get_users<'e>(pool: impl Executor<'e, Database = Postgres>, user_ids: Vec<Uuid>) -> Result<Vec<DBUser>> {
  sqlx::query_as!(
    DBUser,
    r#"
    select * from users where id = any($1);
    "#,
    &user_ids
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_user<'e>(pool: impl Executor<'e, Database = Postgres>, new_user: DBNewUser) -> Result<DBUser> {
  let system_uuid = Uuid::parse_str("11111111-2222-3333-4444-555555555555")?;
  sqlx::query_as!(
      DBUser,
      r#"
      insert into users (auth0id, name, nickname, email, status, org_ids, role_ids, created_by, updated_by)
      values ($1, $2, $3, $4, $5, $6, $7, $8, $8)
      returning *;
      "#,
      new_user.auth0id,
      new_user.name,
      new_user.nickname,
      new_user.email,
      new_user.status,
      &new_user.org_ids,
      &new_user.role_ids,
      system_uuid
    )
    .fetch_one(pool)
    .await
    .map_err(anyhow::Error::from)
}

pub async fn create_user_with_new_org<'e>(pool: PgPool, auth0id: &str, new_user: DBNewUser) -> Result<(DBUser, DBOrg)> {
  let mut tx = pool.begin().await?;

  let mut user = create_user(&mut tx, new_user).await?;

  let new_org = DBNewOrg {
    name: format!("{} personal org", &user.id),
    personal: true,
  };

  let user_org = create_org(&mut tx, auth0id, new_org).await?;

  user.org_ids.push(user_org.id);

  let user_update = DBUpdateUser {
    id: user.id,
    auth0id: None,
    name: None,
    nickname: None,
    email: None,
    status: None,
    org_ids: Some(user.org_ids),
    role_ids: None,
  };

  let user_with_org_id = update_user(&mut tx, auth0id, user_update).await?;

  tx.commit().await?;

  Ok((user_with_org_id, user_org))
}

// Might be a good optimization for the future to use something like:
// "param_1 IS NOT NULL AND param_1 IS DISTINCT FROM column_1" found in this question:
// https://stackoverflow.com/questions/13305878/dont-update-column-if-update-value-is-null
pub async fn update_user<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
  update_user: DBUpdateUser,
) -> Result<DBUser> {
  sqlx::query_as!(
    DBUser,
    r#"
      with _user as (select * from users where auth0id = $1)
      update users
        set
          auth0id = coalesce($3, auth0id),
          name = coalesce($4, name),
          nickname = coalesce($5, nickname),
          email = coalesce($6, email),
          org_ids = coalesce($7, org_ids),
          role_ids = coalesce($8, role_ids),
          status = coalesce($9, status),
          updated_by = coalesce((select id from _user), '11111111-2222-3333-4444-555555555555')
      where id = $2
      returning *;
      "#,
    auth0_user_id,
    update_user.id,
    update_user.auth0id,
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
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}
