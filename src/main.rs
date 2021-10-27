#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate derive_more;
extern crate futures;
extern crate jsonwebtoken as jwt;
extern crate percent_encoding;
extern crate rusoto_core;
extern crate rusoto_ses;
extern crate serde_json;
extern crate serde_qs as qs;
extern crate url;

mod config;

mod extractors;
mod graphql;
mod middleware;
mod routes;
mod services;
mod state;
mod utils;

use crate::graphql::{graphql_routes, schema as graphql_schema};
use crate::routes::general_routes::{get_health, get_info};
use crate::services::auth0_service::Auth0Service;
use crate::services::google_sheets_service::{GoogleSheetsService, OAuthService};
use crate::services::s3_service::{self, S3Service};
use crate::state::State;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use futures::lock::Mutex;
use jsonwebtoken::DecodingKey;
use sqlx::postgres::PgPoolOptions;
use std::{sync::Arc, time::Duration};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  color_backtrace::install();
  openssl_probe::init_ssl_cert_env_vars();
  let config = config::server_config();

  let mut log_builder = env_logger::Builder::new();
  log_builder.parse_filters(&config.logging_directive);
  log_builder.init();

  let pool = PgPoolOptions::new()
    .max_connections(config.database_connection_pool_size)
    .connect_timeout(Duration::new(config.database_connection_timeout_sec, 0))
    .connect(&config.database_url)
    .await
    .expect("Error connecting to database");

  let state = State::new(pool.clone());
  let auth_service = Arc::new(Mutex::new(Auth0Service::new()));
  let oauth_service = Arc::new(Mutex::new(OAuthService::new()));
  let google_sheets_service = Arc::new(Mutex::new(GoogleSheetsService::new()));
  let s3_service = Arc::new(Mutex::new(S3Service::new()));

  let server = HttpServer::new(move || {
    let decoding_key = DecodingKey::from_secret(
      config
        .auth0
        .api_signing_secret
        .as_bytes(),
    )
    .into_static();

    let mut cors = Cors::default()
      .allowed_methods(vec!["GET", "POST", "PATCH", "OPTIONS"])
      .allow_any_header()
      .supports_credentials();
    for origin in &config.allowed_origins {
      cors = cors.allowed_origin(origin);
    }

    App::new()
      .app_data(web::Data::new(state.clone()))
      .app_data(web::Data::new(graphql_schema::create_schema()))
      .app_data(web::Data::new(auth_service.clone()))
      .app_data(web::Data::new(oauth_service.clone()))
      .app_data(web::Data::new(google_sheets_service.clone()))
      .app_data(web::Data::new(s3_service.clone()))
      .app_data(decoding_key)
      .wrap(cors)
      // <response status code> for <path> <remote/proxy ip address> in <seconds>s
      .wrap(actix_web::middleware::Logger::new("%s for %U %a in %Ts"))
      .service(graphql_routes::get_graphql_routes())
      .service(graphql_routes::get_graphql_dev_routes())
      .service(s3_service::get_s3_routes())
      .service(get_health)
      .service(get_info)
    // .service(google_sheets_service::add_data_source)
    // .service(google_sheets_service::exchange_token)
    // .service(google_sheets_service::get_sheets_value)
  });

  let socket_address = format!("0.0.0.0:{}", config.tcp_port);
  server
    .bind(socket_address)?
    .run()
    .await
}
