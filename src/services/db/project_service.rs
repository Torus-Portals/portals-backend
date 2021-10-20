use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DBProject {
  pub id: Uuid,

  pub name: String,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

pub struct DBNewProject {
  pub name: String,
}

pub async fn get_project(pool: impl PgExecutor<'_>, project_id: Uuid) -> Result<DBProject> {
  sqlx::query_as!(
    DBProject,
    "select * from projects where id = $1",
    project_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_projects(pool: impl PgExecutor<'_>, project_ids: &[Uuid]) -> Result<Vec<DBProject>> {
  sqlx::query_as!(DBProject, "select * from projects where id = any($1)", project_ids)
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_auth0_user_projects(pool: impl PgExecutor<'_>, auth0_id: &str) -> Result<Vec<DBProject>> {
  sqlx::query_as!(DBProject, 
    r#"
    with 
      _user as (select * from users where auth0id = $1),
      _user_project as (select * from user_project where user_id = (select id from _user))
    select * from projects where id = (select project_id from _user_project);
    "#,
    auth0_id
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_project(pool: impl PgExecutor<'_>, auth0_id: &str, new_project: DBNewProject) -> Result<DBProject> {
  sqlx::query_as!(
    DBProject,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into projects (name, created_by, updated_by) values ($2, (select id from _user), (select id from _user))
    returning *;
    "#,
    auth0_id,
    new_project.name,
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}
