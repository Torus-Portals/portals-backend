use juniper::{graphql_object, DefaultScalarValue, EmptySubscription, FieldResult, RootNode};
use uuid::Uuid;

// pub mod misc;
pub mod org;
pub mod user;
pub mod role;
pub mod portal;
pub mod portalview;
pub mod structure;
pub mod dimension;
pub mod block;
pub mod cell;

use super::context::GQLContext;
use org::{NewOrg, Org};
use user::{NewUser, User, UpdateUser};
use role::{NewRole, Role};
use portal::{Portal};
use portalview::{NewPortalView, PortalView};
use structure::{Structure};
use dimension::{Dimension};
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

  // User

  async fn user(ctx: &GQLContext, user_id: Uuid) -> FieldResult<User> {
    Query::user_impl(ctx, user_id).await
  }

  async fn current_user(ctx: &GQLContext) -> FieldResult<User> {
    Query::current_user_impl(ctx).await
  }

  // Role

  async fn role(ctx: &GQLContext, role_id: Uuid) -> FieldResult<Role> {
    Query::role_impl(ctx, role_id).await
  }

  // Portal

  async fn portal(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Portal> {
    Query::portal_impl(ctx, portal_id).await
  }

  async fn portals(ctx: &GQLContext) -> FieldResult<Vec<Portal>> {
    Query::user_portals_impl(ctx).await
  }

  // Portal View

  async fn portalviews(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Vec<PortalView>> {
    Query::portalviews_impl(ctx, portal_id).await
  }

  async fn structure(ctx: &GQLContext, structure_id: Uuid) -> FieldResult<Structure> {
    Query::structure_impl(ctx, structure_id).await
  }

  // Dimension

  async fn dimensions(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Vec<Dimension>> {
    Query::dimensions_impl(ctx, portal_id).await
  }

  // Block

  async fn block(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Block> {
    Query::block_impl(ctx, block_id).await
  }

  async fn blocks(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Vec<Block>> {
    Query::blocks_impl(ctx, portal_id).await
  }

  // Cell

  async fn cell(ctx: &GQLContext, cell_id: Uuid) -> FieldResult<Cell> {
    Query::cell_impl(ctx, cell_id).await
  }
}

pub struct Mutation;

#[graphql_object(context = GQLContext)]
impl Mutation {

  // Org
  
  async fn create_org(ctx: &GQLContext, new_org: NewOrg) -> FieldResult<Org> {
    Mutation::create_org_impl(ctx, new_org).await
  }

  // User

  async fn create_user(ctx: &GQLContext, new_user: NewUser) -> FieldResult<User> {
    Mutation::create_user_impl(ctx, new_user).await
  }

  async fn update_user(ctx: &GQLContext, update_user: UpdateUser) -> FieldResult<User> {
    Mutation::update_user_impl(ctx, update_user).await
  }

  // Role

  async fn create_role(ctx: &GQLContext, new_role: NewRole) -> FieldResult<Role> {
    Mutation::create_role_impl(ctx, new_role).await
  }

  // PortalView

  // the space in 'portal_view' is needed so that it shows up as "createPortalView" in GQL Schema
  async fn create_portal_view(ctx: &GQLContext, new_portalview: NewPortalView) -> FieldResult<PortalView> {
    Mutation::create_portalview_impl(ctx, new_portalview).await
  }

  // Structure
}

pub fn create_schema() -> Schema {
  RootNode::new(Query, Mutation, EmptySubscription::new())
}
