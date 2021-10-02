use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::{
  graphql::schema::{
    dimensions::portal_member_dimension::PortalMemberDimension,
    portal::{NewPortal, UpdatePortal},
  },
  services::db::{
    dimension_service::{create_dimensions, DBDimension, DBNewDimension},
    portalview_service::{create_portalview, DBNewPortalView, DBPortalView},
    structure_service::DBStructure,
    user_service::get_user_by_auth0_id,
  },
};

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

#[derive(Serialize, Deserialize)]
pub struct DBUpdatePortal {
  pub id: Uuid,

  pub name: Option<String>,

  pub owner_ids: Option<Vec<Uuid>>,

  pub vendor_ids: Option<Vec<Uuid>>,
}

impl From<UpdatePortal> for DBUpdatePortal {
  fn from(update_portal: UpdatePortal) -> Self {
    DBUpdatePortal {
      id: update_portal.id,
      name: update_portal.name,
      owner_ids: update_portal.owner_ids,
      vendor_ids: update_portal.vendor_ids,
    }
  }
}

pub struct DBPortalParts {
  pub portal: DBPortal,

  pub portal_views: Vec<DBPortalView>,

  pub structures: Vec<DBStructure>,

  pub dimensions: Vec<DBDimension>,
}

pub async fn get_portal(
  pool: impl PgExecutor<'_>,
  portal_id: Uuid,
) -> Result<DBPortal> {
  sqlx::query_as!(DBPortal, "select * from portals where id = $1", portal_id)
    .fetch_one(pool)
    .await
    .map_err(anyhow::Error::from)
}

pub async fn get_portals(
  pool: impl PgExecutor<'_>,
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

pub async fn get_auth0_user_portals(
  pool: impl PgExecutor<'_>,
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

pub async fn create_portal(
  pool: PgPool,
  auth0_user_id: &str,
  new_portal: DBNewPortal,
) -> Result<DBPortalParts> {
  let mut tx = pool.begin().await?;

  let owner_portal_view_pool = pool.clone();
  let vendor_portal_view_pool = pool.clone();

  let user = get_user_by_auth0_id(&mut tx, auth0_user_id).await?;

  let portal = sqlx::query_as!(
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
  .fetch_one(&mut tx)
  .await
  .map_err(anyhow::Error::from)?;

  // Create a default owner and vendor portalview
  let owner_portal_view = create_portalview(
    owner_portal_view_pool,
    auth0_user_id,
    DBNewPortalView {
      portal_id: portal.id,
      name: String::from("Default Owner View"),
      egress: String::from("owner"),
      access: String::from("private"),
    },
  )
  .await?;

  let vendor_portal_view = create_portalview(
    vendor_portal_view_pool,
    auth0_user_id,
    DBNewPortalView {
      portal_id: portal.id,
      name: String::from("Default Vendor View"),
      egress: String::from("vendor"),
      access: String::from("private"),
    },
  )
  .await?;

  let new_dimension = DBNewDimension {
    portal_id: portal.id,
    name: String::from("PortalCreatorUserDimension"),
    dimension_type: String::from("PortalMember"),
    dimension_data: serde_json::to_value(PortalMemberDimension { user_id: user.id })?,
  };

  // TODO: replace this with create_dimension when Tedmund's code lands
  let user_dimensions = create_dimensions(&mut tx, auth0_user_id, vec![new_dimension]).await?;

  tx.commit().await?;

  Ok(DBPortalParts {
    portal,
    portal_views: vec![owner_portal_view.0, vendor_portal_view.0],
    structures: vec![owner_portal_view.1, vendor_portal_view.1],
    dimensions: user_dimensions,
  })
}

pub async fn update_portal(
  pool: impl PgExecutor<'_>,
  // pool: impl PgExecutor<'_>,
  auth0_user_id: &str,
  update_portal: DBUpdatePortal,
) -> Result<DBPortal> {
  sqlx::query_as!(
    DBPortal,
    r#"
    with _user as (select * from users where auth0id = $1)
    update portals
      set
        name = coalesce($3, name),
        owner_ids = coalesce($4, owner_ids),
        vendor_ids = coalesce($5, vendor_ids)
      where id = $2
      returning *;
    "#,
    auth0_user_id,
    update_portal.id,
    update_portal.name,
    update_portal
      .owner_ids
      .as_deref(),
    update_portal
      .vendor_ids
      .as_deref()
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
