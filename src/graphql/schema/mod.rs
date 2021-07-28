use juniper::{graphql_object, DefaultScalarValue, EmptySubscription, FieldResult, RootNode};
use uuid::Uuid;

// pub mod misc;
pub mod org;
pub mod user;
pub mod portal;
pub mod block;
pub mod cell;

use super::context::GQLContext;
use org::{NewOrg, Org};
use user::{User};
use portal::{Portal};
use block::{Block};
use cell::{Cell};

pub type Schema =
  RootNode<'static, Query, Mutation, EmptySubscription<GQLContext>, DefaultScalarValue>;

pub struct Query;

// Have to split the impls up in this weird way do to the graphql macro here:
// https://users.rust-lang.org/t/can-i-add-functions-from-modules-to-a-struct/33791/4
#[graphql_object(context = GQLContext)]
impl Query {
  fn api_version() -> String {
    "1.0".to_string()
  }

  // Org
  async fn org(ctx: &GQLContext, org_id: Uuid) -> FieldResult<Org> {
    Query::org_impl(ctx, org_id).await
  }

  // Orgs, scoped to a User
  #[graphql(description = "Orgs are scoped to the auth0 id of the requesting user")]
  async fn orgs(ctx: &GQLContext) -> FieldResult<Vec<Org>> {
    Query::orgs_impl(ctx).await
  }

  // USer

  async fn user(ctx: &GQLContext, user_id: Uuid) -> FieldResult<User> {
    Query::user_impl(ctx, user_id).await
  }

  async fn current_user(ctx: &GQLContext) -> FieldResult<User> {
    Query::current_user_impl(ctx).await
  }

  // Portal

  async fn portal(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Portal> {
    Query::portal_impl(ctx, portal_id).await
  }

  // Portal View



  // Block

  async fn block(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Block> {
    Query::block_impl(ctx, block_id).await
  }

  // Cell

  async fn cell(ctx: &GQLContext, cell_id: Uuid) -> FieldResult<Cell> {
    Query::cell_impl(ctx, cell_id).await
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
