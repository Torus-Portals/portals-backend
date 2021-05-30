use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLObject, GraphQLUnion, GraphQLEnum};
use serde_json;
use uuid::Uuid;
use std::str::FromStr;
use strum_macros::EnumString;

// use super::Mutation;
use super::Query;
use crate::graphql::context::GQLContext;
use crate::services::db::portal_service::{DBPortal};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct Portal {
  pub id: Uuid,

  pub name: String,

  pub org: Uuid,

  pub owners: Vec<Uuid>,

  pub vendors: Vec<Uuid>,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,
  
  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

impl From <DBPortal> for Portal {
  fn from(db_portal: DBPortal) -> Self {
    Portal {
      id: db_portal.id,
      name: db_portal.name,
      org: db_portal.org,
      owners: db_portal.owners,
      vendors: db_portal.vendors, 
      created_at: db_portal.created_at,
      created_by: db_portal.created_by,
      updated_at: db_portal.updated_at,
      updated_by: db_portal.updated_by,
    }
  }
}

impl Query {
  pub async fn portal_impl(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Portal> {
    ctx.db.get_portal(portal_id).await
    .map(|db_portal| db_portal.into())
    .map_err(FieldError::from)
  }
}