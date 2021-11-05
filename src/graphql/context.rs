use std::sync::Arc;

use futures::lock::Mutex;
use juniper;
use sqlx::PgPool;

use crate::graphql::loaders::org_loader::{get_org_loader, OrgLoader};
use crate::services::auth0_service::Auth0Service;
use crate::services::google_sheets_service::GoogleSheetsService;
use crate::services::s3_service::S3Service;

pub struct GQLContext {
  pub pool: PgPool,
  // Auth related. Might be nice to have the full token here.
  pub auth0_user_id: String,

  // Dataloaders
  pub org_loader: OrgLoader,

  pub auth0_api: Arc<Arc<Mutex<Auth0Service>>>,

  pub google_sheets: Arc<Arc<Mutex<GoogleSheetsService>>>,

  pub s3: Arc<Arc<Mutex<S3Service>>>,
}

impl<'c> juniper::Context for GQLContext {}

impl GQLContext {
  pub fn new(
    pool: PgPool,
    auth0_user_id: String,
    auth0_api: Arc<Arc<Mutex<Auth0Service>>>,
    google_sheets: Arc<Arc<Mutex<GoogleSheetsService>>>,
    s3: Arc<Arc<Mutex<S3Service>>>,
  ) -> Self {
    GQLContext {
      pool: pool.clone(),
      auth0_user_id,
      org_loader: get_org_loader(pool.clone()),
      auth0_api,
      google_sheets,
      s3,
    }
  }
}
