use std::sync::Arc;

use futures::lock::Mutex;
use juniper;
use sqlx::{PgPool};

use crate::graphql::loaders::org_loader::{get_org_loader, OrgLoader};
use crate::graphql::loaders::structure_loader::{get_structure_loader, StructureLoader};
use crate::services::auth0_service::Auth0Service;
use crate::services::google_sheets_service::GoogleSheetsService;
use crate::services::meltano_service::MeltanoService;

pub struct GQLContext {
  pub pool: PgPool,
  // Auth related. Might be nice to have the full token here.
  pub auth0_user_id: String,

  // Dataloaders
  pub org_loader: OrgLoader,

  pub structure_loader: StructureLoader,

  pub auth0_api: Arc<Arc<Mutex<Auth0Service>>>,

  pub google_sheets: Arc<Arc<Mutex<GoogleSheetsService>>>,
  
  pub meltano: Arc<Arc<Mutex<MeltanoService>>>,
}

impl<'c> juniper::Context for GQLContext {}

impl GQLContext {
  pub fn new(
    pool: PgPool,
    auth0_user_id: String,
    auth0_api: Arc<Arc<Mutex<Auth0Service>>>,
    google_sheets: Arc<Arc<Mutex<GoogleSheetsService>>>,
    meltano: Arc<Arc<Mutex<MeltanoService>>>,
  ) -> Self {
    GQLContext {
      pool: pool.clone(),
      auth0_user_id,
      org_loader: get_org_loader(pool.clone()),
      structure_loader: get_structure_loader(pool.clone()),
      auth0_api,
      google_sheets,
      meltano
    }
  }
}
