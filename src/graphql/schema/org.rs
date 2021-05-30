use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult};

use juniper::{GraphQLInputObject, GraphQLObject};

use super::Mutation;
use super::Query;

use crate::graphql::context::GQLContext;
use crate::services::db::org_service::{DBOrg, DBNewOrg};
use uuid::Uuid;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct Org {
  pub id: Uuid,

  pub name: String,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

impl From<DBOrg> for Org {
  fn from(org: DBOrg) -> Self {
    Org {
      id: org.id,
      name: org.name,
      created_at: org.created_at,
      created_by: org.created_by,
      updated_at: org.updated_at,
      updated_by: org.updated_by,
    }
  }
}
#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewOrg {
  pub name: String,
}

impl Query {
  pub async fn orgs_impl(ctx: &GQLContext) -> FieldResult<Vec<Org>> {
    ctx
      .db
      .get_orgs()
      .await
      .map(|orgs| -> Vec<Org> { orgs.into_iter().map(|org| org.into()).collect() })
      .map_err(FieldError::from)
  }

  pub async fn org_impl(ctx: &GQLContext, org_id: Uuid) -> FieldResult<Org> {
    ctx
      .db
      .get_org(org_id)
      .await
      .map(|org| -> Org { org.into() })
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_org_impl(ctx: &GQLContext, new_org: NewOrg) -> FieldResult<Org> {
    ctx
      .db
      .create_org(&ctx.auth0_user_id, DBNewOrg { name: new_org.name })
      .await
      .map(|org| -> Org { org.into() })
      .map_err(FieldError::from)
  }
}
