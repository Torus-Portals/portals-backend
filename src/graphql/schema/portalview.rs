use uuid::Uuid;
use chrono::{DateTime, Utc};
use juniper::{graphql_object, FieldError, FieldResult, GraphQLInputObject};

use super::Mutation;
use super::Query;
use crate::graphql::context::GQLContext;

use crate::services::db::portalview_service::DBPortalView;


// Portal View

#[derive(Debug, Serialize, Deserialize)]
pub struct PortalView {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

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

#[graphql_object(context = GQLContext)]
impl PortalView {
  fn id(&self) -> Uuid {
    self.id
  }

  fn portal_id(&self) -> Uuid {
    self.portal_id
  }

  fn name(&self) -> String {
    self.name.clone()
  }

  fn egress(&self) -> String {
    self.egress.clone()
  }

  fn access(&self) -> String {
    self.access.clone()
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

impl From<DBPortalView> for PortalView {
  fn from(db_portalview: DBPortalView) -> Self {
    PortalView {
      id: db_portalview.id,
      portal_id: db_portalview.portal_id,
      name: db_portalview.name,
      egress: db_portalview.egress,
      access: db_portalview.access,
      created_at: db_portalview.created_at,
      created_by: db_portalview.created_by,
      updated_at: db_portalview.updated_at,
      updated_by: db_portalview.updated_by,
    }
  }
}

impl Query {
  pub async fn portalviews_impl(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Vec<PortalView>> {
    ctx
      .db
      .get_portal_views(portal_id)
      .await
      .map(|db_portalviews| {
        db_portalviews.into_iter().map(|pv| pv.into()).collect()
      })
      .map_err(FieldError::from)
  }
}