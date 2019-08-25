#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde_derive;

extern crate futures;

use actix_web::{ App, HttpServer };
use listenfd::ListenFd;
use dotenv::dotenv;
use db::create_pool;

mod models;
mod routes;
mod schema;
mod db;

fn main() {
  dotenv().ok();

  let pool = create_pool();

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