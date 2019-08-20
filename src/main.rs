#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde_derive;

extern crate futures;

use actix_web::{ web, get, App, HttpResponse, HttpServer, Responder };
use listenfd::ListenFd;
use diesel::prelude::*;
use diesel::r2d2::{ self, ConnectionManager };
use dotenv::dotenv;
use std::env;

mod models;
mod routes;
mod schema;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

fn main() {
  dotenv().ok();

  let db_url = env::var("DATABASE_URL").expect("DATABASE_URL env var must be set.");
  println!("db_url: {}", &db_url);

  let manager = ConnectionManager::<PgConnection>::new(db_url);

  let pool = r2d2::Pool::builder()
    .build(manager)
    .expect("Failed to create db pool");

  let mut listenfd = ListenFd::from_env();
  let mut server = HttpServer::new(move || {
    App::new()
      .data(pool.clone())
      .service(routes::user::get_user_resource())
  });

  server = if let Some(listener) = listenfd.take_tcp_listener(0).unwrap() {
    server.listen(listener).unwrap()
  } else {
    server.bind("127.0.0.1:8088").unwrap()
  };

  server.run().unwrap();
}