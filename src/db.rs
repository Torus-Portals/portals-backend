use diesel::prelude::*;
use diesel::r2d2::{ self, ConnectionManager };
use std::env;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn create_pool() -> Pool {
  let db_url = env::var("DATABASE_URL").expect("DATABASE_URL env var must be set.");

  let manager = ConnectionManager::<PgConnection>::new(db_url);

  r2d2::Pool::builder()
    .build(manager)
    .expect("Failed to create db pool")
}