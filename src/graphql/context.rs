use juniper;
use sqlx::{Pool, Postgres};

pub struct GQLContext {
  // add pool
  pub pool: Pool<Postgres>,
  // add dataloaders
  pub auth0_user_id: String,
}

impl juniper::Context for GQLContext {}

impl GQLContext {
  pub fn new(pool: Pool<Postgres>, auth0_user_id: String) -> Self {
    GQLContext { 
      pool,
      auth0_user_id,
    }
  }
}
