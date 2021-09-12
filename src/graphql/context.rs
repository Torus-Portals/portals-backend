use std::sync::Arc;

use futures::lock::Mutex;
use juniper;
use sqlx::{PgPool};

use crate::graphql::loaders::org_loader::{get_org_loader, OrgLoader};
use crate::graphql::loaders::structure_loader::{get_structure_loader, StructureLoader};
use crate::services::auth0_service::Auth0Service;

pub struct GQLContext {
  pub pool: PgPool,
  // Auth related. Might be nice to have the full token here.
  pub auth0_user_id: String,

  // Dataloaders
  pub org_loader: OrgLoader,

  pub structure_loader: StructureLoader,

  pub auth0_api: Arc<Arc<Mutex<Auth0Service>>>,
}

impl<'c> juniper::Context for GQLContext {}

impl GQLContext {
  pub fn new(
    pool: PgPool,
    auth0_user_id: String,
    auth0_api: Arc<Arc<Mutex<Auth0Service>>>,
  ) -> Self {
    GQLContext {
      pool: pool.clone(),
      auth0_user_id,
      org_loader: get_org_loader(pool.clone()),
      structure_loader: get_structure_loader(pool.clone()),
      auth0_api,
    }
  }
}
