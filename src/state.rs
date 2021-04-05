use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct State {
  pub pool: PgPool,
}

impl State {
  pub fn new(pool: PgPool) -> Self {
    State {
      pool
    }
  }
}