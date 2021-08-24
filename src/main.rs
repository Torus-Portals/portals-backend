#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_qs as qs;
extern crate derive_more;

// #[macro_use]
// extern crate json_payload_derive;

extern crate futures;

extern crate jsonwebtoken as jwt;

extern crate percent_encoding;
extern crate url;

extern crate rusoto_core;
extern crate rusoto_ses;

extern crate base64;

mod graphql;
mod middleware;
mod extractors;
mod services;
mod state;
mod utils;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{get, web, App, Error, HttpResponse, HttpServer};
// use base64::encode;
use dotenv::dotenv;
use futures::lock::Mutex;
use jsonwebtoken::DecodingKey;
use listenfd::ListenFd;
use sqlx::postgres::PgPoolOptions;

use crate::graphql::{graphql_routes, schema as graphql_schema};
use crate::state::State;

use crate::services::auth0_service::Auth0Service;

#[get("/health")]
pub async fn get_health() -> Result<HttpResponse, Error> {
  Ok(HttpResponse::Ok().body(String::from("Hello from the other side!")))
}

#[actix_web::main]
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

  let db_url = std::env::var("DATABASE_URL").expect("Unable to get DATABASE_URL env var.");
  println!("db_url: {}", db_url);

  let host = std::env::var("PORTALS_MAIN_HOST").expect("Unable to get PORTALS_MAIN_HOST env var.");
  println!("host: {}", host);

  println!("Creating db pool");
  let pool = PgPoolOptions::new()
    .max_connections(5) // TODO: env var this
    .connect(&db_url)
    .await
    .unwrap();

  let state = State::new(pool.clone());
  let auth_service = Arc::new(Mutex::new(Auth0Service::new()));

  let mut listenfd = ListenFd::from_env();
  let mut server = HttpServer::new(move || {
    let client_secret =
      std::env::var("AUTH0_API_SIGNING_SECRET").expect("Unable to get AUTH0_API_SIGNING_SECRET.");

    // let b64_client_secret = encode(&client_secret);

    let decoding_key = DecodingKey::from_secret(client_secret.as_bytes()).into_static();

    App::new()
      .app_data(web::Data::new(state.clone()))
      .app_data(web::Data::new(graphql_schema::create_schema()))
      .app_data(web::Data::new(auth_service.clone()))
      .app_data(decoding_key)
      .wrap(
        Cors::default()
          .allowed_origin("http://localhost:8088") // TODO: env var this
          .allowed_origin("https://local.portals-dev.rocks") // TODO: env var this
          .allowed_methods(vec!["GET", "POST", "PATCH", "OPTIONS"])
          .allow_any_header()
          .supports_credentials(),
      )
      .service(graphql_routes::get_graphql_routes())
      .service(graphql_routes::get_graphql_dev_routes())
      .service(get_health)
  });

  server = if let Some(listener) = listenfd
    .take_tcp_listener(0)
    .unwrap()
  {
    println!("re-listening...");
    server
      .listen(listener)
      .unwrap()
  } else {
    println!("Binding for the very first time!");
    server
      .bind(host)
      .unwrap()
  };

  println!("Server created");
  server.run().await
}
