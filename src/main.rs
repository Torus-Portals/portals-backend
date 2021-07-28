#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_qs as qs;

#[macro_use]
extern crate json_payload_derive;

extern crate futures;

extern crate jsonwebtoken as jwt;

extern crate percent_encoding;
extern crate url;

extern crate rusoto_core;
extern crate rusoto_ses;

extern crate base64;

mod graphql;
mod middleware;
mod models;
mod queries;
mod routes;
mod schema;
mod services;
mod state;
mod utils;

use actix_cors::Cors;
use actix_web::{get, middleware as actix_middleware, App, Error, HttpResponse, HttpServer};
use dotenv::dotenv;
use jsonwebtoken::DecodingKey;
use listenfd::ListenFd;
use sqlx::postgres::PgPoolOptions;
use base64::encode;

use crate::graphql::{graphql_routes, schema as graphql_schema};
use crate::state::State;

// NOTE: I don't know if this will always be length of 270, but this is working for now..
pub static KEY: [u8; 270] = *include_bytes!("../auth0.der");

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

  // let db_url = std::env::var("DATABASE_URL").unwrap();
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

  let mut listenfd = ListenFd::from_env();
  let mut server = HttpServer::new(move || {
    let client_secret = std::env::var("AUTH0_API_SIGNING_SECRET").expect("Unable to get AUTH0_API_SIGNING_SECRET.");

    // let b64_client_secret = encode(&client_secret);

    let decoding_key = DecodingKey::from_secret(client_secret.as_bytes()).into_static();

    App::new()
      .data(state.clone())
      .data(graphql_schema::create_schema())
      // .app_data(decoding_key.clone())
      .app_data(decoding_key)
      // .app_data(DecodingKey::from_secret(b64_client_secret.as_bytes()))
      // .app_data(DecodingKey::from_secret())
      // .wrap(actix_middleware::Logger::new("%r %s size:%b time in ms:%D"))
      .wrap(
        Cors::default()
          .allowed_origin("http://localhost:8088") // TODO: env var this
          .allowed_origin("https://local.portals-dev.rocks:3000") // TODO: env var this
          .allowed_methods(vec!["GET", "POST", "PATCH", "OPTIONS"])
          .allow_any_header()
          .supports_credentials(),
      )
      .service(graphql_routes::get_graphql_routes())
      .service(graphql_routes::get_graphql_dev_routes())
      .service(get_health)
  });

  server = if let Some(listener) = listenfd.take_tcp_listener(0).unwrap() {
    println!("re-listening...");
    server.listen(listener).unwrap()
  } else {
    println!("Binding for the very first time!");
    server.bind(host).unwrap()
  };

  println!("Server created");
  server.run().await
}
