#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

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
mod services;
mod utils;

use middleware::auth::Auth;
use middleware::auth::AuthDer;

use crate::routes::{ users, orgs, portals, portalviews, blocks, cells };

fn load_key(filename: &str) -> Vec<u8> {
  let mut buffer = Vec::<u8>::new();
  let mut file = File::open(filename).unwrap();
  file.read_to_end(&mut buffer).unwrap();
  buffer
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  dotenv().ok();

  let pool = create_pool();

  let key = load_key("auth0.der");

  let mut listenfd = ListenFd::from_env();
  let mut server = HttpServer::new(move || {
    App::new()
      .data(pool.clone())
      .data(AuthDer(key.clone()))
      .wrap(Auth)
      .wrap(actix_middleware::Logger::default())
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
      .service(cells::get_cell_routes())
  });

  server = if let Some(listener) = listenfd.take_tcp_listener(0).unwrap() {
    server.listen(listener).unwrap()
  } else {
    server.bind("127.0.0.1:8088").unwrap()
  };

  server.run().await
}