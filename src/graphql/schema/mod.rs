use juniper::{graphql_object, EmptySubscription, RootNode, DefaultScalarValue};
use uuid::Uuid;

pub mod query;
pub mod mutation;
pub mod org;
pub mod misc;

use query::Query;
use mutation::Mutation;
use super::context::GQLContext;

use crate::models::org::{NewOrg, Org};

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<GQLContext>, DefaultScalarValue>;

// Have to split the impls up in this weird way do to the graphql macro here:
// https://users.rust-lang.org/t/can-i-add-functions-from-modules-to-a-struct/33791/4
#[graphql_object(context = GQLContext)]
impl Query {
    fn api_version() -> String {
        "1.0".to_string()
    }

    // Org
    async fn orgs(ctx: &GQLContext) -> Vec<Org> {
      Query::orgs_impl(ctx).await
    }

    async fn org(ctx: &GQLContext, id: Uuid) -> Org {
      Query::org_impl(ctx, id).await
    }
}

#[graphql_object(context = GQLContext)]
impl Mutation {
  async fn create_org(ctx: &GQLContext, new_org: NewOrg) -> Org {
    Mutation::create_org_impl(ctx, new_org).await
  }
}

pub fn create_schema() -> Schema {
  RootNode::new(Query, Mutation, EmptySubscription::new())
}
