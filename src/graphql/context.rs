use std::sync::Arc;

use crate::services::db::DB;
use futures::lock::Mutex;
use juniper;
use sqlx::PgPool;

use crate::graphql::loaders::org_loader::{get_org_loader, OrgLoader};
use crate::services::auth0_service::Auth0Service;

pub struct GQLContext {
  pub pool: PgPool,
  // Auth related. Might be nice to have the full token here.
  pub auth0_user_id: String,

  pub db: DB,

  // Dataloaders
  pub org_loader: OrgLoader,

  pub auth0_api: Arc<Arc<Mutex<Auth0Service>>>,
}

impl juniper::Context for GQLContext {}

impl GQLContext {
  pub fn new(
    pool: PgPool,
    auth0_user_id: String,
    auth0_api: Arc<Arc<Mutex<Auth0Service>>>,
  ) -> Self {
    let db = DB::new(pool.clone());

    GQLContext {
      pool: pool.clone(),
      auth0_user_id,
      db: db.clone(),
      org_loader: get_org_loader(db),
      auth0_api,
    }
  }
}
