use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult};

use juniper::{GraphQLInputObject, GraphQLObject};

use super::Mutation;
use super::Query;

use super::user::{UpdateUser, User};

use crate::graphql::context::GQLContext;
use crate::services::db::org_service::{create_org, get_org, DBNewOrg, DBOrg};
use crate::services::db::user_service::{get_user_by_auth0_id, update_user};
use uuid::Uuid;

#[derive(GraphQLObject, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Org {
  pub id: Uuid,

  pub name: String,

  pub personal: bool,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl From<DBOrg> for Org {
  fn from(org: DBOrg) -> Self {
    Org {
      id: org.id,
      name: org.name,
      personal: org.personal,
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
    let user = get_user_by_auth0_id(&ctx.pool, &ctx.auth0_user_id).await?;

    let user_org_ids = user.org_ids.clone();

    let orgs_by_id = ctx
      .org_loader
      .load_many(user.org_ids.into())
      .await;

    let orgs = orgs_by_id
      .into_iter()
      .fold(Vec::new(), |mut acc, (id, o)| {
        if user_org_ids.contains(&id) {
          acc.push(o)
        };
        acc
      });

    Ok(orgs)
  }

  pub async fn org_impl(ctx: &GQLContext, org_id: Uuid) -> FieldResult<Org> {
    get_org(&ctx.pool, org_id)
      .await
      .map(|org| -> Org { org.into() })
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_org_impl(ctx: &GQLContext, new_org: NewOrg) -> FieldResult<Org> {
    let created_org = create_org(
      &ctx.pool,
      &ctx.auth0_user_id,
      DBNewOrg {
        name: new_org.name,
        personal: false,
      },
    )
    .await
    .map(|org| -> Org { org.into() })
    .map_err(FieldError::from)?;

    let user = get_user_by_auth0_id(&ctx.pool, &ctx.auth0_user_id)
      .await
      .map(|db_user| -> User { db_user.into() })
      .map_err(FieldError::from)?;

    let mut new_org_ids = user.org_ids.clone();
    new_org_ids.push(created_org.id);

    let user_patch = UpdateUser {
      id: user.id,
      auth0id: None,
      name: None,
      nickname: None,
      email: None,
      user_status: None,
      org_ids: Some(new_org_ids),
      role_ids: None,
    };

    // Add the user who created the org to the org.
    update_user(&ctx.pool, &ctx.auth0_user_id, user_patch.into())
      .await
      .map_err(FieldError::from)?;

    Ok(created_org)
  }
}
