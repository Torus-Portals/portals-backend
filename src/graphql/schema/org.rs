use juniper::{FieldError, FieldResult};

use super::Query;
use super::Mutation;


use crate::models::org::Org;
use crate::{graphql::context::GQLContext, models::org::NewOrg};
use uuid::Uuid;

impl Query {
  pub async fn orgs_impl(ctx: &GQLContext) -> FieldResult<Vec<Org>> {
    ctx.db.get_orgs().await.map_err(FieldError::from)
  }

  pub async fn org_impl(ctx: &GQLContext, org_id: Uuid) -> FieldResult<Org> {
    ctx.db.get_org(org_id).await.map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_org_impl(ctx: &GQLContext, new_org: NewOrg) -> FieldResult<Org> {
    ctx
      .db
      .create_org(&ctx.auth0_user_id, new_org)
      .await
      .map_err(FieldError::from)
  }
}
