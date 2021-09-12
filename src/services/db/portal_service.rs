use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

use crate::graphql::schema::portal::NewPortal;

#[derive(Debug, Serialize, Deserialize)]
pub struct DBPortal {
  pub id: Uuid,

  pub name: String,

  pub org_id: Uuid,

  pub owner_ids: Vec<Uuid>,

  pub vendor_ids: Vec<Uuid>,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct DBNewPortal {
  pub org_id: Uuid,

  pub name: String,

  pub owner_ids: Vec<Uuid>,

  pub vendor_ids: Vec<Uuid>,
}

impl From<NewPortal> for DBNewPortal {
  fn from(new_portal: NewPortal) -> Self {
    DBNewPortal {
      org_id: new_portal.org_id,
      name: new_portal.name,
      owner_ids: new_portal.owner_ids,
      vendor_ids: new_portal.vendor_ids,
    }
  }
}

pub async fn get_portal<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  portal_id: Uuid,
) -> Result<DBPortal> {
  sqlx::query_as!(DBPortal, "select * from portals where id = $1", portal_id)
    .fetch_one(pool)
    .await
    .map_err(anyhow::Error::from)
}

pub async fn get_portals<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  portal_ids: Vec<Uuid>,
) -> Result<Vec<DBPortal>> {
  sqlx::query_as!(
    DBPortal,
    "select * from portals where id = any($1)",
    &portal_ids
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_auth0_user_portals<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
) -> Result<Vec<DBPortal>> {
  sqlx::query_as!(
    DBPortal,
    r#"
    with _user as (select * from users where auth0id = $1)
    select * from portals where
    (select id from _user) = any(owner_ids) or
    (select id from _user) = any(vendor_ids);
    "#,
    auth0_user_id
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_portal<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0_user_id: &str,
  new_portal: DBNewPortal,
) -> Result<DBPortal> {
  sqlx::query_as!(
    DBPortal,
    r#"
      with _user as (select * from users where auth0id = $1)
      insert into portals (
        name,
        org_id,
        owner_ids,
        vendor_ids,
        created_by,
        updated_by
      ) values (
        $2,
        $3,
        $4,
        $5,
        (select id from _user),
        (select id from _user)
      ) returning *
      "#,
    auth0_user_id,
    new_portal.name,
    new_portal.org_id,
    &new_portal.owner_ids,
    &new_portal.vendor_ids
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn delete_portal<'e>(pool: PgPool, portal_id: Uuid) -> Result<i32> {
  let mut tx = pool.begin().await?;

  let portal = get_portal(&mut tx, portal_id).await?;

  dbg!(portal);

  tx.commit().await?;

  Ok(1)
}
