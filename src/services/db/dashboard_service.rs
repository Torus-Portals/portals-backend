use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{Executor, PgExecutor};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DBDashboard {
  pub id: Uuid,

  pub name: String,

  pub project_id: Uuid,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

pub struct DBNewDashboard {
  pub name: String,

  pub project_id: Uuid,
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

pub async fn get_dashboards(pool: impl PgExecutor<'_>, dashboard_ids: &[Uuid]) -> Result<Vec<DBDashboard>> {
  sqlx::query_as!(DBDashboard, "select * from dashboards where id = any($1)", dashboard_ids)
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_project_dashboards(pool: impl PgExecutor<'_>, project_id: Uuid) -> Result<Vec<DBDashboard>> {
  sqlx::query_as!(DBDashboard, "select * from dashboards where project_id = $1", project_id)
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_dashboards(pool: impl PgExecutor<'_>, auth0_id: &str, new_dashboard: DBNewDashboard) -> Result<DBDashboard> {
  sqlx::query_as!(
    DBDashboard,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into dashboards (name, created_by, updated_by) values ($2, (select id from _user), (select id from _user))
    returning *;
    "#,
    auth0_id,
    new_dashboard.name,
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}
