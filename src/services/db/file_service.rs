use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::graphql::schema::file::NewFile;

pub struct DBFile {
  pub id: Uuid,

  pub name: String,

  pub key: String,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,

  pub deleted_at: Option<DateTime<Utc>>,

  pub deleted_by: Option<Uuid>,
}

pub struct DBNewFile {
  pub name: String,

  pub key: String,
}

impl From<NewFile> for DBNewFile {
  fn from(new_file: NewFile) -> Self {
    DBNewFile {
      name: new_file.name,
      key: new_file.key,
    }
  }
}

pub async fn get_file(pool: impl PgExecutor<'_>, file_id: Uuid) -> Result<DBFile> {
  sqlx::query_as!(
    DBFile,
    r#"
    select
      id,
      name,
      key,
      created_at,
      created_by,
      updated_at,
      updated_by,
      deleted_at,
      deleted_by
    from 
      files
    where 
      id = $1 and deleted_at is null
    "#,
    file_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_file(
  pool: impl PgExecutor<'_>,
  auth0_user_id: &str,
  new_file: DBNewFile,
) -> Result<DBFile> {
  dbg!(&auth0_user_id);

  sqlx::query_as!(
    DBFile,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into files (name, key, created_by, updated_by)
    values ($2, $3, (select id from _user), (select id from _user))
    returning *;
    "#,
    auth0_user_id,
    new_file.name,
    new_file.key
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}
