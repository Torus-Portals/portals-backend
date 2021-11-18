use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::graphql::schema::dashboard::{NewDashboard, UpdateDashboard};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DBDashboard {
  pub id: Uuid,

  pub name: String,

  pub project_id: Uuid,

  pub page_ids: Vec<Uuid>,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

pub struct DBNewDashboard {
  pub name: String,

  pub project_id: Uuid,
}

impl From<NewDashboard> for DBNewDashboard {
  fn from(new_dashboard: NewDashboard) -> Self {
    DBNewDashboard {
      name: new_dashboard.name,
      project_id: new_dashboard.project_id,
    }
  }
}

pub struct DBUpdateDashboard {
  pub id: Uuid,

  pub name: Option<String>,

  pub project_id: Option<Uuid>,

  pub page_ids: Option<Vec<Uuid>>,
}

impl From<UpdateDashboard> for DBUpdateDashboard {
  fn from(update_dashboard: UpdateDashboard) -> Self {
    DBUpdateDashboard {
      id: update_dashboard.id,
      name: update_dashboard.name,
      project_id: update_dashboard.project_id,
      page_ids: update_dashboard.page_ids,
    }
  }
}

pub async fn get_dashboard(pool: impl PgExecutor<'_>, dashboard_id: Uuid) -> Result<DBDashboard> {
  sqlx::query_as!(
    DBDashboard,
    "select * from dashboards where id = $1",
    dashboard_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn _get_dashboards(
  pool: impl PgExecutor<'_>,
  dashboard_ids: &[Uuid],
) -> Result<Vec<DBDashboard>> {
  sqlx::query_as!(
    DBDashboard,
    "select * from dashboards where id = any($1)",
    dashboard_ids
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_project_dashboards(
  pool: impl PgExecutor<'_>,
  project_ids: &[Uuid],
) -> Result<Vec<DBDashboard>> {
  sqlx::query_as!(
    DBDashboard,
    "select * from dashboards where project_id = any($1)",
    project_ids
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_dashboard(
  pool: impl PgExecutor<'_>,
  auth0_id: &str,
  new_dashboard: DBNewDashboard,
) -> Result<DBDashboard> {
  // TODO: Would be nice to be able to create a default starter page for a new dashboard.

  sqlx::query_as!(
    DBDashboard,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into dashboards (name, project_id, created_by, updated_by)
    values ($2, $3, (select id from _user), (select id from _user))
    returning *;
    "#,
    auth0_id,
    new_dashboard.name,
    new_dashboard.project_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn update_dashboard(
  pool: impl PgExecutor<'_>,
  auth0_id: &str,
  updated_dashboard: DBUpdateDashboard,
) -> Result<DBDashboard> {
  sqlx::query_as!(
    DBDashboard,
    r#"
    with _user as (select * from users where auth0id = $1)
    update dashboards
    set
      name = coalesce($3, name),
      project_id = coalesce($4, project_id),
      page_ids = coalesce($5, page_ids),
      updated_by = (select id from _user)
    where id = $2
    returning *;
    "#,
    auth0_id,
    updated_dashboard.id,
    updated_dashboard.name,
    updated_dashboard.project_id,
    updated_dashboard
      .page_ids
      .as_deref()
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn remove_page_from_dashboard(
  pool: impl PgExecutor<'_>,
  auth0_id: &str,
  page_id: Uuid,
  dashboard_id: Uuid,
) -> Result<DBDashboard> {
  sqlx::query_as!(
    DBDashboard,
    r#"
    with _user as (select * from users where auth0id = $1)
    update dashboards
    set 
      page_ids = array_remove(page_ids, $2),
      updated_by = (select id from _user)
    where id = $3
    returning *;
    "#,
    auth0_id,
    page_id,
    dashboard_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}
