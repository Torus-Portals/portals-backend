use sqlx::{PgPool};

#[derive(Debug, Clone)]
pub struct DB {
  pub pool: PgPool,
}

impl DB {
  pub fn new(pool: PgPool) -> Self {
    DB { pool }
  }
}