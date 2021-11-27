use std::str::FromStr;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::graphql::schema::policy::{GrantTypes, NewPolicy, UpdatePolicy};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DBPolicy {
  pub id: Uuid,

  pub resource_id: Uuid,

  pub policy_type: String,

  pub permission_type: String,

  pub grant_type: String,

  pub user_ids: Vec<Uuid>,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBNewPolicy {
  pub resource_id: Uuid,

  pub policy_type: String,

  pub permission_type: String,

  pub grant_type: String,

  pub user_ids: Vec<Uuid>,
}

impl From<NewPolicy> for DBNewPolicy {
  fn from(new_policy: NewPolicy) -> Self {
    DBNewPolicy {
      resource_id: new_policy.resource_id,
      policy_type: new_policy.policy_type.to_string(),
      permission_type: new_policy.permission_type.to_string(),
      grant_type: new_policy.grant_type.to_string(),
      user_ids: new_policy.user_ids,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBUpdatePolicy {
  pub resource_id: Uuid,

  pub policy_type: String,

  pub permission_type: String,

  pub grant_type: String,

  pub user_ids: Vec<Uuid>,
}

impl From<UpdatePolicy> for DBUpdatePolicy {
  fn from(update_policy: UpdatePolicy) -> Self {
    DBUpdatePolicy {
      resource_id: update_policy.resource_id,
      policy_type: update_policy.policy_type.to_string(),
      permission_type: update_policy.permission_type.to_string(),
      grant_type: update_policy.grant_type.to_string(),
      user_ids: update_policy.user_ids,
    }
  }
}

pub async fn create_policy(
  pool: impl PgExecutor<'_>,
  auth0_user_id: &str,
  new_policy: DBNewPolicy,
) -> Result<i32> {
  let grants = match GrantTypes::from_str(&new_policy.grant_type)? {
    GrantTypes::All => GrantTypes::iter()
      .skip(1)
      .map(|g| g.to_string())
      .collect::<Vec<String>>(),
    _ => vec![new_policy.grant_type],
  };

  sqlx::query!(
      r#"
      with _user as (select * from users where auth0id = $1)
      insert into policies (resource_id, policy_type, permission_type, grant_type, user_ids, created_by, updated_by)
      select $2, $3, $4, *, $6, (select id from _user), (select id from _user)
      from unnest($5::text[]);
      "#,
      auth0_user_id,
      new_policy.resource_id,
      new_policy.policy_type,
      new_policy.permission_type,
      &grants,
      &new_policy.user_ids
    )
    .execute(pool)
    .await
    .map(|qr| qr.rows_affected() as i32)
    .map_err(anyhow::Error::from)
}

pub async fn update_policy(
  pool: impl PgExecutor<'_>,
  auth0_user_id: &str,
  update_policy: DBUpdatePolicy,
) -> Result<DBPolicy> {
  sqlx::query_as!(
    DBPolicy,
    r#"
    with _user as (select * from users where auth0id = $1)
    update policies
      set
        user_ids = coalesce($2, user_ids)
      where resource_id = $3
      and policy_type = $4
      and permission_type = $5
      and grant_type = $6
      returning *;
    "#,
    auth0_user_id,
    &update_policy.user_ids,
    update_policy.resource_id,
    update_policy.policy_type,
    update_policy.permission_type,
    update_policy.grant_type
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn check_permission(
  pool: impl PgExecutor<'_>,
  resource_id: Uuid,
  user_id: Uuid,
  grant_type: String,
) -> Result<bool> {
  let db_policy = sqlx::query_as!(
    DBPolicy,
    r#"
    select * from policies
    where resource_id = $1 and user_ids @> $2 and grant_type = $3;
    "#,
    resource_id,
    &vec![user_id],
    grant_type,
  )
  .fetch_optional(pool)
  .await
  .map_err(anyhow::Error::from)?;

  Ok(db_policy.is_some())
}

pub async fn user_permissions(pool: impl PgExecutor<'_>, user_id: Uuid) -> Result<Vec<DBPolicy>> {
  sqlx::query_as!(
    DBPolicy,
    r#"
    select * from policies
    where user_ids @> $1
    "#,
    &vec![user_id],
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn user_resource_perms(
  pool: impl PgExecutor<'_>,
  user_id: Uuid,
  resource_id: Uuid,
) -> Result<Vec<DBPolicy>> {
  sqlx::query_as!(
    DBPolicy,
    r#"
    select * from policies
    where user_ids @> $1 and resource_id = $2
    "#,
    &vec![user_id],
    resource_id,
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn resources_perms(
  pool: impl PgExecutor<'_>,
  resource_ids: Vec<Uuid>,
) -> Result<Vec<DBPolicy>> {
  sqlx::query_as!(
    DBPolicy,
    r#"
    select * from policies
    where resource_id = any($1)
    "#,
    &resource_ids,
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}