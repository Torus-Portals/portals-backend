use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult};

use juniper::{GraphQLInputObject, GraphQLObject};

use super::Mutation;
use super::Query;

use crate::graphql::context::GQLContext;
use crate::services::db::org_service::{DBNewOrg, DBOrg};
use uuid::Uuid;

#[derive(GraphQLObject, Debug, Serialize, Deserialize, Clone)]
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
    let user = ctx.db.get_user_by_auth0_id(&ctx.auth0_user_id).await?;
    let user_orgs = user.orgs.clone();

    let orgs_by_id = ctx.org_loader.load_many(user.orgs.into()).await;

    let orgs = orgs_by_id
      .into_iter()
      .fold(Vec::new(), |mut acc, (id, o)| {
        if user_orgs.contains(&id) { acc.push(o) };
        acc
      });

    Ok(orgs)
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
