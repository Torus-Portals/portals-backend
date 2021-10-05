use juniper::{graphql_object, EmptySubscription, FieldResult, RootNode};
use uuid::Uuid;

pub mod block;
pub mod blocks;
pub mod cell;
pub mod cells;
pub mod dimension;
pub mod dimensions;
pub mod integration;
pub mod integrations;
pub mod org;
pub mod portal;
pub mod portalview;
pub mod role;
pub mod structure;
pub mod user;

use self::integrations::google_sheets::GoogleSheetsAuthorization;

use super::context::GQLContext;
use block::{Block, BlockParts, UpdateBlock};
use blocks::{
  basic_table_block::NewBasicTableBlock, integration_block::NewIntegrationBlock,
  owner_text_block::NewOwnerTextBlock, vendor_text_block::NewVendorTextBlock,
};
use cell::{Cell, UpdateCell};
use dimension::{Dimension, NewDimension};
use org::{NewOrg, Org};
use portal::{Portal, PortalParts, NewPortal, PortalInviteParams, UpdatePortal};
use portalview::{NewPortalView, PortalView, PortalViewParts};
use role::{NewRole, Role};
use structure::{Structure, UpdateStructure};
use user::{NewUser, UpdateUser, User};
use integration::{Integration, NewIntegration};
use integrations::google_sheets::GoogleSheetsRedirectURI;

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<GQLContext>>;
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

  async fn users(ctx: &GQLContext, user_ids: Vec<Uuid>) -> FieldResult<Vec<User>> {
    Query::users_impl(ctx, user_ids).await
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

  async fn portals_by_ids(ctx: &GQLContext, portal_ids: Vec<Uuid>) -> FieldResult<Vec<Portal>> {
    Query::portals_by_ids_impl(ctx, portal_ids).await
  }

  // Portal View

  async fn portalviews(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Vec<PortalView>> {
    Query::portalviews_impl(ctx, portal_id).await
  }

  // Structure

  async fn structure(ctx: &GQLContext, structure_id: Uuid) -> FieldResult<Structure> {
    Query::structure_impl(ctx, structure_id).await
  }

  async fn structures(ctx: &GQLContext, structure_ids: Vec<Uuid>) -> FieldResult<Vec<Structure>> {
    Query::structures_impl(ctx, structure_ids).await
  }

  // Block

  async fn block(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Block> {
    Query::block_impl(ctx, block_id).await
  }

  async fn blocks(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Vec<Block>> {
    Query::blocks_impl(ctx, portal_id).await
  }

  async fn integration_block_options(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Integration> {
    Query::integration_block_options_impl(ctx, block_id).await
  }

  // Dimension

  async fn dimension(ctx: &GQLContext, dimension_id: Uuid) -> FieldResult<Dimension> {
    Query::dimension_impl(ctx, dimension_id).await
  }

  async fn dimensions(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Vec<Dimension>> {
    Query::dimensions_impl(ctx, portal_id).await
  }

  // Cell

  async fn cell(ctx: &GQLContext, cell_id: Uuid) -> FieldResult<Cell> {
    Query::cell_impl(ctx, cell_id).await
  }

  async fn cells_any_dimensions(
    ctx: &GQLContext,
    dimension_ids: Vec<Uuid>,
  ) -> FieldResult<Vec<Cell>> {
    Query::cells_any_dimensions_impl(ctx, dimension_ids).await
  }

  async fn cells_all_dimensions(
    ctx: &GQLContext,
    dimension_ids: Vec<Uuid>,
  ) -> FieldResult<Vec<Cell>> {
    Query::cells_all_dimensions_impl(ctx, dimension_ids).await
  }

  // Integration

  async fn integration(ctx: &GQLContext, integration_id: Uuid) -> FieldResult<Integration> {
    Query::integration_impl(ctx, integration_id).await
  }

  async fn integrations(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Vec<Integration>> {
    Query::integrations_impl(ctx, portal_id).await
  }

  // Specific Integrations

  async fn google_sheets_redirect_uri() -> FieldResult<GoogleSheetsRedirectURI> {
    Query::google_sheets_redirect_uri_impl().await
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

  // Portal

  async fn create_portal(ctx: &GQLContext, new_portal: NewPortal) -> FieldResult<PortalParts> {
    Mutation::create_portal_impl(ctx, new_portal).await
  }

  async fn delete_portal(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<i32> {
    Mutation::delete_portal_impl(ctx, portal_id).await
  }

  async fn update_portal(ctx: &GQLContext, portal_update: UpdatePortal) -> FieldResult<Portal> {
    Mutation::update_portal_impl(ctx, portal_update).await
  }

  async fn invite_user_to_portal(
    ctx: &GQLContext,
    portal_invite_params: PortalInviteParams,
  ) -> FieldResult<PortalParts> {
    Mutation::invite_user_to_portal_impl(ctx, portal_invite_params).await
  }

  // PortalView

  // the space in 'portal_view' is needed so that it shows up as "createPortalView" in GQL Schema
  async fn create_portal_view(
    ctx: &GQLContext,
    new_portalview: NewPortalView,
  ) -> FieldResult<PortalViewParts> {
    Mutation::create_portalview_impl(ctx, new_portalview).await
  }

  // Structure

  async fn update_structure(
    ctx: &GQLContext,
    update_structure: UpdateStructure,
  ) -> FieldResult<Structure> {
    Mutation::update_structure_impl(ctx, update_structure).await
  }

  // Block

  // Not using for the time being.
  // async fn create_block(ctx: &GQLContext, new_block: NewBlock) -> FieldResult<Block> {
  //   Mutation::create_block(ctx, new_block).await
  // }

  async fn update_block(ctx: &GQLContext, update_block: UpdateBlock) -> FieldResult<Block> {
    Mutation::update_block_impl(ctx, update_block).await
  }

  async fn delete_block(ctx: &GQLContext, block_id: Uuid) -> FieldResult<i32> {
    Mutation::delete_block(ctx, block_id).await
  }

  async fn delete_blocks(ctx: &GQLContext, block_ids: Vec<Uuid>) -> FieldResult<i32> {
    Mutation::delete_blocks(ctx, block_ids).await
  }

  async fn create_basic_table(
    ctx: &GQLContext,
    new_basic_table_block: NewBasicTableBlock,
  ) -> FieldResult<Block> {
    Mutation::create_basic_table_impl(ctx, new_basic_table_block).await
  }

  async fn create_owner_text_block(
    ctx: &GQLContext,
    new_owner_text_block: NewOwnerTextBlock,
  ) -> FieldResult<BlockParts> {
    Mutation::create_owner_text_block_impl(ctx, new_owner_text_block).await
  }

  async fn create_vendor_text_block(
    ctx: &GQLContext,
    new_vendor_text_block: NewVendorTextBlock,
  ) -> FieldResult<Block> {
    Mutation::create_vendor_text_block_impl(ctx, new_vendor_text_block).await
  }

  async fn create_integration_block(
    ctx: &GQLContext,
    new_integration_block: NewIntegrationBlock,
  ) -> FieldResult<BlockParts> {
    Mutation::create_integration_block_impl(ctx, new_integration_block).await
  }

  // Dimension

  async fn create_dimensions(
    ctx: &GQLContext,
    new_dimensions: Vec<NewDimension>,
  ) -> FieldResult<Vec<Dimension>> {
    Mutation::create_dimensions_impl(ctx, new_dimensions).await
  }

  // Cell

  async fn update_cell(ctx: &GQLContext, update_cell: UpdateCell) -> FieldResult<Cell> {
    Mutation::update_cell_impl(ctx, update_cell).await
  }

  // Integration

  async fn create_integration(
    ctx: &GQLContext,
    new_integration: NewIntegration,
  ) -> FieldResult<Integration> {
    Mutation::create_integration_impl(ctx, new_integration).await
  }

  // async fn delete_integration(ctx: &GQLContext, integration_id: Uuid) -> FieldResult<i32> {
  //   Mutation::delete_integration(ctx, integration_id).await
  // }

  // Specific Integrations

  async fn authorize_google_sheets(ctx: &GQLContext, auth: GoogleSheetsAuthorization) -> FieldResult<bool> {
    Mutation::authorize_google_sheets_impl(ctx, auth).await
  }
}

pub fn create_schema() -> Schema {
  RootNode::new(Query, Mutation, EmptySubscription::new())
}
