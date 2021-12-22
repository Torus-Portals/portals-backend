use anyhow::{Result};
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor};
use uuid::Uuid;

use crate::graphql::schema::{file::NewFile};

pub struct DBFile {
  pub id: Uuid,

  pub project_id: Uuid,

  pub name: String,

  pub key: Uuid,

  pub extension: String,

  pub version: i32,

  pub size: i32,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,

  pub deleted_at: Option<DateTime<Utc>>,

  pub deleted_by: Option<Uuid>,
}

pub struct DBNewFile {
  pub id: Uuid,

  pub project_id: Uuid,

  pub name: String,

  pub key: Uuid,

  pub extension: String,

  pub size: i32,
}

impl From<NewFile> for DBNewFile {
  fn from(new_file: NewFile) -> Self {
    DBNewFile {
      id: new_file.id,
      project_id: new_file.project_id,
      name: new_file.name,
      key: new_file.key,
      extension: new_file.extension,
      size: new_file.size,
    }
  }
}

pub async fn get_file(pool: impl PgExecutor<'_>, file_id: Uuid) -> Result<DBFile> {
  sqlx::query_as!(
    DBFile,
    r#"
    select * from files where id = $1 and deleted_at is null
    "#,
    file_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_files(pool: impl PgExecutor<'_>, file_ids: Vec<Uuid>) -> Result<Vec<DBFile>> {
  sqlx::query_as!(
    DBFile,
    r#"
    select * from files where id = any($1) and deleted_at is null
    "#,
    &file_ids
  )
  .fetch_all(pool)
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
    with _user as (select * from users where auth0id = $1),
    _last_version as (select coalesce(max(version), 0) as version from files where key = $5)
    insert into files (id, project_id, name, key, extension, version, size, created_by, updated_by)
    values ($2, $3, $4, $5, $6, (select version from _last_version) + 1, $7, (select id from _user), (select id from _user))
    returning *;
    "#,
    auth0_user_id,
    new_file.id,
    new_file.project_id,
    new_file.name,
    new_file.key,
    new_file.extension,
    new_file.size
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}
