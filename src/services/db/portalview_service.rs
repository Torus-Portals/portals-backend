use anyhow::Result;
use chrono::{DateTime, Utc};

use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

use crate::{
  graphql::schema::{portalview::NewPortalView, structure::GridStructure},
  services::db::structure_service::{create_structure, DBStructure, DBNewStructure},
};

#[derive(Debug, Serialize)]
pub struct DBPortalView {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub structure_id: Uuid,

  pub name: String,

  pub egress: String,

  pub access: String,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

pub struct DBNewPortalView {
  pub portal_id: Uuid,

  pub name: String,

  pub egress: String,

  pub access: String,
}

impl From<NewPortalView> for DBNewPortalView {
  fn from(new_portalview: NewPortalView) -> Self {
    DBNewPortalView {
      portal_id: new_portalview.portal_id,
      name: new_portalview.name,
      egress: new_portalview.egress,
      access: new_portalview.access,
    }
  }
}

pub async fn get_portalviews<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  portal_id: Uuid,
) -> Result<Vec<DBPortalView>> {
  sqlx::query_as!(
    DBPortalView,
    r#"
      select * from portalviews where portal_id = $1
      "#,
    portal_id
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn create_portalview(
  pool: PgPool,
  auth0_user_id: &str,
  new_portalview: DBNewPortalView,
) -> Result<(DBPortalView, DBStructure)> {
  let mut tx = pool.begin().await?;

  // generate the structure in here since every portalview should have a structure
  let structure_data = GridStructure::new();
  let structure = create_structure(
    &mut tx,
    auth0_user_id,
    DBNewStructure {
      structure_type: String::from("Grid"),
      structure_data: serde_json::to_value(structure_data)
        .ok()
        .unwrap(),
    },
  )
  .await?;

  let portal_view = sqlx::query_as!(
    DBPortalView,
    r#"
      with _user as (select * from users where auth0id = $1)
      insert into portalviews (
        portal_id,
        name,
        egress,
        access,
        structure_id,
        created_by,
        updated_by
      ) values (
        $2,
        $3,
        $4,
        $5,
        $6,
        (select id from _user),
        (select id from _user)
      ) returning *

      "#,
    auth0_user_id,
    new_portalview.portal_id,
    new_portalview.name,
    new_portalview.egress,
    new_portalview.access,
    structure.id
  )
  .fetch_one(&mut tx)
  .await
  .map_err(anyhow::Error::from)?;

  tx.commit().await?;

  Ok((portal_view, structure))
}

pub async fn delete_portal_portalviews<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  portal_id: Uuid,
) -> Result<i32> {
  sqlx::query!(
    r#"
    delete from portalviews where portal_id = $1;
    "#,
    portal_id
  )
  .execute(pool)
  .await
  .map(|qr| qr.rows_affected() as i32)
  .map_err(anyhow::Error::from)
}
