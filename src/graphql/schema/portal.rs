use chrono::{DateTime, Utc};
use juniper::{graphql_object, FieldError, FieldResult, GraphQLEnum, GraphQLObject, GraphQLUnion};
use serde_json;
use std::str::FromStr;
use strum_macros::EnumString;
use uuid::Uuid;

// use super::Mutation;
use super::Query;
use crate::graphql::context::GQLContext;
use crate::services::db::portal_service::DBPortal;

#[derive(Debug, Serialize, Deserialize)]
pub struct Portal {
  pub id: Uuid,

  pub name: String,

  pub org: Uuid,

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

  fn org(&self) -> Uuid {
    self.org
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
      org: db_portal.org,
      owner_ids: db_portal.owner_ids,
      vendor_ids: db_portal.vendor_ids,
      created_at: db_portal.created_at,
      created_by: db_portal.created_by,
      updated_at: db_portal.updated_at,
      updated_by: db_portal.updated_by,
    }
  }
}

impl Query {
  pub async fn portal_impl(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Portal> {
    ctx
      .db
      .get_portal(portal_id)
      .await
      .map(|db_portal| db_portal.into())
      .map_err(FieldError::from)
  }

  // Get all portals associated to a user
  pub async fn user_portals_impl(ctx: &GQLContext) -> FieldResult<Vec<Portal>> {
    ctx.db.get_auth0_user_portals(&ctx.auth0_user_id)
    .await
    .map(|db_portals| {
      db_portals.into_iter().map(|p| p.into()).collect()
    })
    .map_err(FieldError::from)
  }
}
