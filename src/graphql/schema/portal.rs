use chrono::{DateTime, Utc};
use juniper::{graphql_object, FieldError, FieldResult, GraphQLInputObject};

use uuid::Uuid;

use super::Mutation;
use super::Query;

use crate::graphql::context::GQLContext;
use crate::services::db::portal_service::create_portal;
use crate::services::db::portal_service::delete_portal;
use crate::services::db::portal_service::get_auth0_user_portals;
use crate::services::db::portal_service::get_portal;
use crate::services::db::portal_service::DBNewPortal;
use crate::services::db::portal_service::DBPortal;
use crate::services::db::portal_service::get_portals;
use crate::services::db::portalview_service::DBNewPortalView;
use crate::services::db::portalview_service::create_portalview;

#[derive(Debug, Serialize, Deserialize)]
pub struct Portal {
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

#[graphql_object(context = GQLContext)]
impl Portal {
  fn id(&self) -> Uuid {
    self.id
  }

  fn name(&self) -> String {
    self.name.clone()
  }

  fn org_id(&self) -> Uuid {
    self.org_id
  }

  fn owner_ids(&self) -> Vec<Uuid> {
    self
      .owner_ids
      .clone()
  }

  fn vendor_ids(&self) -> Vec<Uuid> {
    self
      .vendor_ids
      .clone()
  }

  fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  fn created_by(&self) -> Uuid {
    self.created_by
  }

  fn updated_at(&self) -> DateTime<Utc> {
    self.updated_at
  }

  fn updated_by(&self) -> Uuid {
    self.updated_by
  }
}

impl From<DBPortal> for Portal {
  fn from(db_portal: DBPortal) -> Self {
    Portal {
      id: db_portal.id,
      name: db_portal.name,
      org_id: db_portal.org_id,
      owner_ids: db_portal.owner_ids,
      vendor_ids: db_portal.vendor_ids,
      created_at: db_portal.created_at,
      created_by: db_portal.created_by,
      updated_at: db_portal.updated_at,
      updated_by: db_portal.updated_by,
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewPortal {
  pub name: String,

  pub org_id: Uuid,

  pub owner_ids: Vec<Uuid>,

  pub vendor_ids: Vec<Uuid>,
}

impl From<DBNewPortal> for NewPortal {
  fn from(db_new_portal: DBNewPortal) -> Self {
    NewPortal {
      org_id: db_new_portal.org_id,
      name: db_new_portal.name,
      owner_ids: db_new_portal.owner_ids,
      vendor_ids: db_new_portal.vendor_ids,
    }
  }
}

impl Query {
  pub async fn portal_impl(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Portal> {
    get_portal(&ctx.pool, portal_id)
      .await
      .map(|db_portal| db_portal.into())
      .map_err(FieldError::from)
  }

  // Get all portals associated to a user
  pub async fn user_portals_impl(ctx: &GQLContext) -> FieldResult<Vec<Portal>> {
    get_auth0_user_portals(&ctx.pool, &ctx.auth0_user_id)
      .await
      .map(|db_portals| {
        db_portals
          .into_iter()
          .map(|p| p.into())
          .collect()
      })
      .map_err(FieldError::from)
  }

  pub async fn portals_by_ids_impl(
    ctx: &GQLContext,
    portal_ids: Vec<Uuid>,
  ) -> FieldResult<Vec<Portal>> {
    get_portals(&ctx.pool, portal_ids)
      .await
      .map(|db_portals| {
        db_portals
          .into_iter()
          .map(|p| p.into())
          .collect()
      })
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_portal_impl(ctx: &GQLContext, new_portal: NewPortal) -> FieldResult<Portal> {
    let portal: Portal = create_portal(&ctx.pool, &ctx.auth0_user_id, new_portal.into())
      .await
      .map(|db_portal| db_portal.into())
      .map_err(FieldError::from)?;

    // Create a default owner and vendor portalview
    create_portalview(
      &ctx.pool,
      &ctx.auth0_user_id,
      DBNewPortalView {
        portal_id: portal.id,
        name: String::from("Default Owner View"),
        egress: String::from("owner"),
        access: String::from("private"),
      },
    )
    .await?;

    create_portalview(
      &ctx.pool,
      &ctx.auth0_user_id,
      DBNewPortalView {
        portal_id: portal.id,
        name: String::from("Default Owner View"),
        egress: String::from("vendor"),
        access: String::from("private"),
      },
    )
    .await?;

    Ok(portal)
  }

  pub async fn delete_portal_impl(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<i32> {
    let local_pool = ctx.pool.clone();
    delete_portal(local_pool, portal_id).await
    .map_err(FieldError::from)
  }
}
