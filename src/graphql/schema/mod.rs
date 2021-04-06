use juniper::{graphql_object, EmptySubscription, RootNode, DefaultScalarValue, FieldResult};
use uuid::Uuid;

pub mod org;
pub mod misc;

use super::context::GQLContext;

use crate::models::org::{NewOrg, Org};

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<GQLContext>, DefaultScalarValue>;

pub struct Query;

// Have to split the impls up in this weird way do to the graphql macro here:
// https://users.rust-lang.org/t/can-i-add-functions-from-modules-to-a-struct/33791/4
#[graphql_object(context = GQLContext)]
impl Query {
    fn api_version() -> String {
        "1.0".to_string()
    }

    // Org
    async fn org(ctx: &GQLContext, id: Uuid) -> FieldResult<Org> {
      Query::org_impl(ctx, id).await
    }

    async fn orgs(ctx: &GQLContext) -> FieldResult<Vec<Org>> {
      Query::orgs_impl(ctx).await
    }

}

pub struct Mutation;

#[graphql_object(context = GQLContext)]
impl Mutation {
  async fn create_org(ctx: &GQLContext, new_org: NewOrg) -> FieldResult<Org> {
    Mutation::create_org_impl(ctx, new_org).await
  }
}

pub fn create_schema() -> Schema {
  RootNode::new(Query, Mutation, EmptySubscription::new())
}
