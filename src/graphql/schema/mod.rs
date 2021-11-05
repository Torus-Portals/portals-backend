use chrono::{DateTime, Utc};
use juniper::{graphql_object, EmptySubscription, FieldResult, RootNode};
use uuid::Uuid;

pub mod block;
pub mod blocks;
pub mod connection;
pub mod connection_content;
pub mod dashboard;
pub mod integration;
pub mod integrations;
pub mod org;
pub mod page;
pub mod project;
pub mod role;
pub mod source;
pub mod sourcequeries;
pub mod sourcequery;
pub mod sources;
pub mod s3;
pub mod user;

use self::integrations::google_sheets::GoogleSheetsAuthorization;

use super::context::GQLContext;
use block::{Block, NewBlock, UpdateBlock};
use connection::{Connection, NewConnection, UpdateConnection};
use connection_content::{ConnectionContent};
use source::{NewSource, Source};
use dashboard::{Dashboard, NewDashboard, UpdateDashboard};
use integration::{Integration, NewIntegration};
use integrations::google_sheets::GoogleSheetsRedirectURI;
use org::{NewOrg, Org};
use page::{NewPage, Page, UpdatePage};
use project::{NewProject, Project, ProjectParts};
use role::{NewRole, Role};
use source::{PossibleSource, PossibleSourceInput};
use user::{NewUser, UpdateUser, User};

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
  
  async fn user_by_email(ctx: &GQLContext, user_email: String) -> FieldResult<User> {
    Query::user_by_email_impl(ctx, user_email).await
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

  // Project

  async fn project(ctx: &GQLContext, project_id: Uuid) -> FieldResult<Project> {
    Query::project_impl(ctx, project_id).await
  }

  async fn projects(ctx: &GQLContext) -> FieldResult<Vec<Project>> {
    Query::projects_impl(ctx).await
  }
  
  async fn share_project(ctx: &GQLContext, project_id: Uuid, user_ids: Vec<Uuid>) -> FieldResult<i32> {
    Query::share_project_impl(ctx, project_id, user_ids).await
  }

  // Dashboard

  async fn dashboard(ctx: &GQLContext, dashboard_id: Uuid) -> FieldResult<Dashboard> {
    Query::dashboard_impl(ctx, dashboard_id).await
  }

  async fn dashboards(ctx: &GQLContext, project_id: Uuid) -> FieldResult<Vec<Dashboard>> {
    Query::dashboards_impl(ctx, project_id).await
  }
  
  async fn share_dashboard(ctx: &GQLContext, dashboard_id: Uuid, user_ids: Vec<Uuid>) -> FieldResult<i32> {
    Query::share_dashboard_impl(ctx, dashboard_id, user_ids).await
  }

  // Page

  async fn page(ctx: &GQLContext, page_id: Uuid) -> FieldResult<Page> {
    Query::page_impl(ctx, page_id).await
  }

  async fn pages(ctx: &GQLContext, dashboard_id: Uuid) -> FieldResult<Vec<Page>> {
    Query::pages_impl(ctx, dashboard_id).await
  }

  // Block

  async fn block(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Block> {
    Query::block_impl(ctx, block_id).await
  }

  async fn blocks(ctx: &GQLContext, block_ids: Vec<Uuid>) -> FieldResult<Vec<Block>> {
    Query::blocks_impl(ctx, block_ids).await
  }

  async fn page_blocks(ctx: &GQLContext, page_id: Uuid) -> FieldResult<Vec<Block>> {
    Query::page_blocks_impl(ctx, page_id).await
  }

  // async fn integration_block_options(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Integration> {
  //   Query::integration_block_options_impl(ctx, block_id).await
  // }

  // Connection

  async fn connections(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Vec<Connection>> {
    Query::connections_impl(ctx, block_id).await
  }

  async fn connection_content(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Vec<ConnectionContent>> {
    Query::connection_content_impl(ctx, block_id).await
  }

  // Source

  async fn possible_sources(
    ctx: &GQLContext,
    input: PossibleSourceInput,
  ) -> FieldResult<Vec<PossibleSource>> {
    Query::possible_sources_impl(ctx, input).await
  }

  // Integration

  async fn integration(ctx: &GQLContext, integration_id: Uuid) -> FieldResult<Integration> {
    Query::integration_impl(ctx, integration_id).await
  }

  async fn integrations(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Vec<Integration>> {
    Query::integrations_impl(ctx, portal_id).await
  }

  // Specific Integrations

  async fn google_sheets_redirect_uri(state: String) -> FieldResult<GoogleSheetsRedirectURI> {
    Query::google_sheets_redirect_uri_impl(state).await
  }
  
  // S3

  async fn s3_upload_presigned_url(ctx: &GQLContext, bucket: String, key: String) -> FieldResult<String> {
    Query::s3_upload_presigned_url_impl(ctx, bucket, key).await
  }

  async fn s3_download_presigned_url(ctx: &GQLContext, bucket: String, key: String) -> FieldResult<String> {
    Query::s3_download_presigned_url_impl(ctx, bucket, key).await
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

  // Project

  async fn create_project(ctx: &GQLContext, new_project: NewProject) -> FieldResult<Project> {
    Mutation::create_project_impl(ctx, new_project).await
  }

  async fn add_user_to_project(ctx: &GQLContext, user_email: String, project_id: Uuid) -> FieldResult<Project> {
    Mutation::add_user_to_project_impl(ctx, user_email, project_id).await
  }

  // Dashboard

  async fn create_dashboard(
    ctx: &GQLContext,
    new_dashboard: NewDashboard,
  ) -> FieldResult<Dashboard> {
    Mutation::create_dashboard_impl(ctx, new_dashboard).await
  }

  async fn update_dashboard(
    ctx: &GQLContext,
    update_dashboard: UpdateDashboard,
  ) -> FieldResult<Dashboard> {
    Mutation::update_dashboard_impl(ctx, update_dashboard).await
  }

  // Page

  async fn create_page(ctx: &GQLContext, new_page: NewPage) -> FieldResult<Page> {
    Mutation::create_page_impl(ctx, new_page).await
  }

  async fn update_page(ctx: &GQLContext, updated_page: UpdatePage) -> FieldResult<Page> {
    Mutation::update_page_impl(ctx, updated_page).await
  }

  async fn delete_page(ctx: &GQLContext, page_id: Uuid) -> FieldResult<DateTime<Utc>> {
    Mutation::delete_page_impl(ctx, page_id).await
  }

  // Block

  async fn create_block(ctx: &GQLContext, new_block: NewBlock) -> FieldResult<Block> {
    Mutation::create_block(ctx, new_block).await
  }

  async fn update_block(ctx: &GQLContext, update_block: UpdateBlock) -> FieldResult<Block> {
    Mutation::update_block_impl(ctx, update_block).await
  }

  async fn delete_block(ctx: &GQLContext, block_id: Uuid) -> FieldResult<DateTime<Utc>> {
    Mutation::delete_block(ctx, block_id).await
  }

  async fn delete_blocks(ctx: &GQLContext, block_ids: Vec<Uuid>) -> FieldResult<i32> {
    Mutation::delete_blocks(ctx, block_ids).await
  }

  // Connection

  async fn create_connection(
    ctx: &GQLContext,
    new_connection: NewConnection,
  ) -> FieldResult<Connection> {
    Mutation::create_connection_impl(ctx, new_connection).await
  }

  async fn update_connection(
    ctx: &GQLContext,
    update_connection: UpdateConnection,
  ) -> FieldResult<Connection> {
    Mutation::update_connection_impl(ctx, update_connection).await
  }

  // Source

  async fn create_source(ctx: &GQLContext, new_source: NewSource) -> FieldResult<Source> {
    Mutation::create_source_impl(ctx, new_source).await
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

  async fn authorize_google_sheets(
    ctx: &GQLContext,
    auth: GoogleSheetsAuthorization,
  ) -> FieldResult<bool> {
    Mutation::authorize_google_sheets_impl(ctx, auth).await
  }
}

pub fn create_schema() -> Schema {
  RootNode::new(Query, Mutation, EmptySubscription::new())
}
