use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::{graphql::schema::{page::{Grid, NewPage, UpdatePage}, policy::{GrantTypes, NewPolicy, PermissionTypes, PolicyTypes}}, services::db::{policy_service::{check_permission, create_policy}, user_service::get_user_by_auth0_id}};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DBPage {
  pub id: Uuid,

  pub name: String,
  
  pub project_id: Uuid,

  pub dashboard_id: Uuid,

  pub grid: serde_json::Value,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,

  pub deleted_at: Option<DateTime<Utc>>,

  pub deleted_by: Option<Uuid>,
}

pub struct DBNewPage {
  pub name: String,

  pub project_id: Uuid,

  pub dashboard_id: Uuid,
}

impl From<NewPage> for DBNewPage {
  fn from(new_page: NewPage) -> Self {
    DBNewPage {
      name: new_page.name,
      project_id: new_page.project_id,
      dashboard_id: new_page.dashboard_id,
    }
  }
}

pub struct DBUpdatePage {
  pub id: Uuid,

  pub name: Option<String>,

  pub project_id: Option<Uuid>,

  pub dashboard_id: Option<Uuid>,

  pub grid: Option<serde_json::Value>,
}

impl From<UpdatePage> for DBUpdatePage {
  fn from(update_page: UpdatePage) -> Self {
    let grid = serde_json::to_value(&update_page.grid).ok();

    DBUpdatePage {
      id: update_page.id,
      name: update_page.name,
      project_id: update_page.project_id,
      dashboard_id: update_page.dashboard_id,
      grid,
    }
  }
}

pub async fn get_page(pool: impl PgExecutor<'_>, page_id: Uuid) -> Result<DBPage> {
  sqlx::query_as!(
    DBPage,
    "select * from pages where id = $1 and deleted_at = null",
    page_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn _get_pages(pool: impl PgExecutor<'_>, page_ids: &[Uuid]) -> Result<Vec<DBPage>> {
  sqlx::query_as!(
    DBPage,
    "select * from pages where id = any($1) and deleted_at = null",
    page_ids
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_dashboard_pages(
  pool: impl PgExecutor<'_>,
  dashboard_id: Uuid,
) -> Result<Vec<DBPage>> {
  sqlx::query_as!(
    DBPage,
    "select * from pages where dashboard_id = $1",
    dashboard_id
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_page(
  pool: PgPool,
  auth0_id: &str,
  new_page: DBNewPage,
) -> Result<DBPage> {
  let mut tx = pool.begin().await?;
  let grid = serde_json::to_value(Grid::new())?;
  let user = get_user_by_auth0_id(&mut tx, auth0_id).await?;

  if !check_permission(
    &mut tx,
    new_page.dashboard_id,
    user.id,
    GrantTypes::Create.to_string(),
  )
  .await?
  {
    return Err(anyhow!(
      "Current user does not have permission to create page"
    ));
  }

  let page = sqlx::query_as!(
    DBPage,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into pages (name, project_id, dashboard_id, grid, created_by, updated_by)
    values ($2, $3, $4, $5, (select id from _user), (select id from _user))
    returning *;
    "#,
    auth0_id,
    new_page.name,
    new_page.project_id,
    new_page.dashboard_id,
    grid
  )
  .fetch_one(&mut tx)
  .await
  .map_err(anyhow::Error::from)?;
    
  let new_page_policy = NewPolicy {
    resource_id: page.id,
    policy_type: PolicyTypes::PagePolicy,
    permission_type: PermissionTypes::BlockPermission,
    grant_type: GrantTypes::All,
    user_ids: vec![user.id],
  };
  create_policy(&mut tx, auth0_id, new_page_policy.into()).await?;

  tx.commit().await?;

  Ok(page)
}

pub async fn update_page(
  pool: impl PgExecutor<'_>,
  auth0_id: &str,
  new_page: DBUpdatePage,
) -> Result<DBPage> {
  sqlx::query_as!(
    DBPage,
    r#"
    with _user as (select * from users where auth0id = $1)
    update pages
      set 
        name = coalesce($3, name),
        project_id = coalesce($4, project_id),
        dashboard_id = coalesce($5, dashboard_id),
        grid = coalesce($6, grid),
        updated_by = (select id from _user)
    where id = $2
    returning *;
    "#,
    auth0_id,
    new_page.id,
    new_page.name,
    new_page.project_id,
    new_page.dashboard_id,
    new_page.grid,
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn delete_page(pool: impl PgExecutor<'_>, auth0_id: &str, page_id: Uuid) -> Result<DateTime<Utc>> {
  sqlx::query_as!(
    DBPage,
    r#"
    with _user as (select * from users where auth0id = $1)
    update pages
      set
        deleted_at = now(),
        deleted_by = (select id from _user)
    where id = $2
    returning *;
    "#,
    auth0_id,
    page_id
  )
  .fetch_one(pool)
  .await
  .map(|db_p|db_p.deleted_at.unwrap())
  .map_err(anyhow::Error::from)
}
