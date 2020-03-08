#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_qs as qs;

#[macro_use]
extern crate json_payload_derive;

extern crate futures;

extern crate jsonwebtoken as jwt;

extern crate url;
extern crate percent_encoding;

use actix_web::{ middleware as actix_middleware, App, HttpServer };
use actix_cors::Cors;
use listenfd::ListenFd;
use dotenv::dotenv;
use db::create_pool;

use std::io::prelude::*;
use std::fs::File;

mod models;
mod routes;
mod schema;
mod middleware;
mod db;
mod queries;
mod services;
mod utils;

use middleware::auth::Auth;
use middleware::auth::AuthDer;

use crate::routes::{
  users,
  orgs,
  portals,
  portalviews,
  blocks,
  dimensions,
  cells
};

// TODO: Handle missing file better. 
fn load_key(filename: &str) -> Vec<u8> {
  let mut buffer = Vec::<u8>::new();
  let mut file = File::open(filename).unwrap();
  file.read_to_end(&mut buffer).unwrap();
  buffer
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  println!("starting server");
  if cfg!(feature = "local_dev") {
    println!("In local development!");
    dotenv().ok();
  }

  println!("loading logging vars");
  std::env::set_var("RUST_LOG", "my_errors=debug,actix_web=info");

  std::env::set_var("RUST_BACKTRACE", "1");

  env_logger::init();

  // let db_url = std::env::var("DATABASE_URL").unwrap();
  let db_url = std::env::var("DATABASE_URL")
    .expect("Unable to get DATABASE_URL env var.");
  println!("db_url: {}", db_url);

  let host = std::env::var("PORTALS_MAIN_HOST")
    .expect("Unable to get PORTALS_MAIN_HOST env var.");
  println!("host: {}", host);

  println!("Creating db pool");
  let pool = create_pool();

  println!("Creating db pool");
  let key = load_key("auth0.der");
  println!("have key");

  let mut listenfd = ListenFd::from_env();
  let mut server = HttpServer::new(move || {
    App::new()
      .data(pool.clone())
      .data(AuthDer(key.clone()))
      // .wrap(actix_middleware::Logger::default())
      .wrap(actix_middleware::Logger::new("%r %s size:%b time:%D"))
      .wrap(Auth)
      .wrap(
        Cors::new()
          .allowed_origin("https://local.torus-dev.rocks:3001")
          .allowed_methods(vec!["GET", "POST", "PATCH"]).finish()
      )
      .service(users::get_user_routes())
      .service(orgs::get_org_routes())
      .service(portals::get_portal_routes())
      .service(portalviews::get_portalview_routes())
      .service(blocks::get_block_routes())
      .service(dimensions::get_dimension_routes())
      .service(cells::get_cell_routes())
  });

  server = if let Some(listener) = listenfd.take_tcp_listener(0).unwrap() {
    println!("re-listening...");
    server.listen(listener).unwrap()
  } else {
    println!("Binding for the very first time!");
    server.bind(host).unwrap()
    // server.bind("127.0.0.1:8088").unwrap()
  };

  println!("Server created");
  server.run().await
}