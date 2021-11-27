use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::{
  graphql::schema::{
    dashboard::{NewDashboard, UpdateDashboard},
    policy::{GrantTypes, NewPolicy, PermissionTypes, PolicyTypes},
  },
  services::db::{
    policy_service::{check_permission, create_policy},
    project_service::{add_user_to_project, get_auth0_user_projects},
    user_service::get_user_by_auth0_id,
  },
};

use super::project_service::get_project;

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

  pub deleted_at: Option<DateTime<Utc>>,

  pub deleted_by: Option<Uuid>,
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

// pub async fn get_project_dashboards(
//   pool: impl PgExecutor<'_>,
//   project_id: Uuid,
// ) -> Result<Vec<DBDashboard>> {
//   sqlx::query_as!(
//     DBDashboard,
//     "select * from dashboards where project_id = $1",
//     project_id
//   )
//   .fetch_all(pool)
//   .await
//   .map_err(anyhow::Error::from)
// }

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

pub async fn add_user_to_dashboard(
  pool: impl PgExecutor<'_>,
  auth0_id: &str,
  user_id: Uuid,
  dashboard_id: Uuid,
) -> Result<i32> {
  sqlx::query!(
    r#"
  with _user as (select * from users where auth0id = $1)
  insert into user_access (user_id, object_type, object_id, created_by, updated_by)
  values ($2, 'Dashboard', $3, (select id from _user), (select id from _user))
  "#,
    auth0_id,
    user_id,
    dashboard_id,
  )
  .execute(pool)
  .await
  .map(|qr| qr.rows_affected() as i32)
  .map_err(anyhow::Error::from)
}

pub async fn add_user_to_dashboards(
  pool: impl PgExecutor<'_>,
  auth0_id: &str,
  user_id: Uuid,
  dashboard_ids: &[Uuid],
) -> Result<i32> {
  sqlx::query!(
    r#"
  with _user as (select * from users where auth0id = $1)
  insert into user_access (user_id, object_type, object_id, created_by, updated_by)
  select $2, 'Dashboard', *, (select id from _user), (select id from _user)
  from unnest($3::uuid[])
  "#,
    auth0_id,
    user_id,
    dashboard_ids,
  )
  .execute(pool)
  .await
  .map(|qr| qr.rows_affected() as i32)
  .map_err(anyhow::Error::from)
}

pub async fn create_dashboard(
  pool: PgPool,
  auth0_id: &str,
  new_dashboard: DBNewDashboard,
) -> Result<DBDashboard> {
  let mut tx = pool.begin().await?;
  let user = get_user_by_auth0_id(&mut tx, auth0_id).await?;

  // TODO: check if user has permission to create a dashboard in current project?
  if !check_permission(
    &mut tx,
    new_dashboard.project_id,
    user.id,
    GrantTypes::Create.to_string(),
  )
  .await?
  {
    return Err(anyhow!(
      "Current user does not have permission to create dashboard"
    ));
  }

  // TODO: Would be nice to be able to create a default starter page for a new dashboard.
  // TODO: check if user has permission to add dashboard to current project
  let dashboard = sqlx::query_as!(
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
  .fetch_one(&mut tx)
  .await
  .map_err(anyhow::Error::from)?;

  // let user_projects = get_auth0_user_projects(&mut tx, auth0_id).await?;
  // if !user_projects
  //   .into_iter()
  //   .map(|db_project| db_project.id)
  //   .any(|id| id == dashboard.project_id)
  // {
  //   add_user_to_project(&mut tx, auth0_id, user.id, dashboard.project_id).await?;
  // }
  // add_user_to_dashboard(&mut tx, auth0_id, user.id, dashboard.id).await?;
  let new_dashboard_policy = NewPolicy {
    resource_id: dashboard.id,
    policy_type: PolicyTypes::DashboardPolicy,
    permission_type: PermissionTypes::PagePermission,
    grant_type: GrantTypes::All,
    user_ids: vec![user.id],
  };
  create_policy(&mut tx, auth0_id, new_dashboard_policy.into()).await?;

  tx.commit().await?;

  Ok(dashboard)
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

pub async fn share_dashboard(
  pool: PgPool,
  auth0_id: &str,
  dashboard_id: Uuid,
  user_ids: Vec<Uuid>,
) -> Result<i32> {
  let mut tx = pool.begin().await?;
  let mut res = 0;

  // Adds user to containing Project as well -- but not other dashboards
  let dashboard = get_dashboard(&mut tx, dashboard_id).await?;
  let project = get_project(&mut tx, dashboard.project_id).await?;
  for user_id in user_ids {
    res += add_user_to_dashboard(&mut tx, auth0_id, user_id, dashboard_id).await?;
    res += add_user_to_project(&mut tx, auth0_id, user_id, project.id).await?;
  }

  tx.commit().await?;

  Ok(res)
}
