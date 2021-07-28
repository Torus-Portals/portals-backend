use crate::services::db::DB;
use juniper;
use sqlx::PgPool;

use crate::graphql::loaders::org_loader::{get_org_loader, OrgLoader};

pub struct GQLContext {
  // add pool
  pub pool: PgPool,
  // Auth related. Might be nice to have the full token here.
  pub auth0_user_id: String,

  pub db: DB,

  // Dataloaders
  pub org_loader: OrgLoader,
}

impl juniper::Context for GQLContext {}

impl GQLContext {
  pub fn new(pool: PgPool, auth0_user_id: String) -> Self {
    let db = DB::new(pool.clone());
    // let loaders = Loaders::New(db);

    GQLContext {
      pool: pool.clone(),
      auth0_user_id,
      db: db.clone(),
      org_loader: get_org_loader(db),
    }
  }
}
