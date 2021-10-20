use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{Executor, PgExecutor};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DBPage {
  pub id: Uuid,

  pub name: String,

  pub project_id: Uuid,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

pub struct DBNewPage {
  pub name: String,

  pub project_id: Uuid,
}

pub async fn get_page(pool: impl PgExecutor<'_>, page_id: Uuid) -> Result<DBPage> {
  sqlx::query_as!(
    DBPage,
    "select * from pages where id = $1",
    page_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_pages(pool: impl PgExecutor<'_>, page_ids: &[Uuid]) -> Result<Vec<DBPage>> {
  sqlx::query_as!(DBPage, "select * from pages where id = any($1)", page_ids)
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_page(pool: impl PgExecutor<'_>, auth0_id: &str, new_page: DBNewPage) -> Result<DBPage> {
  sqlx::query_as!(
    DBPage,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into pages (name, project_id, created_by, updated_by) values ($2, $3, (select id from _user), (select id from _user))
    returning id, name, project_id, created_at, created_by, updated_at, updated_by
    "#,
    auth0_id,
    new_page.name,
    new_page.project_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}
