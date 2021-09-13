use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::{env, fs::File};

static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Deserialize)]
pub struct Config {
  /// https://docs.rs/env_logger/0.9.0/env_logger/index.html#enabling-logging.
  pub logging_directive: String,
  pub tcp_port: u16,
  pub allowed_origins: Vec<String>,
  pub graceful_shutdown_timeout_sec: u64,
  pub max_payload_size_bytes: usize,
  pub database_connection_pool_size: u32,
  pub database_connection_timeout_sec: u64,
  pub database_url: String,
  pub auth0: Auth0,
}

#[derive(Deserialize)]
pub struct Auth0 {
  pub client_id: String,
  pub client_secret: String,
  pub token_endpoint: String,
  pub api_endpoint: String,
  pub api_signing_secret: String,
}

fn init() -> Config {
  if let Ok(path) = env::var("CONFIG_PATH") {
    let config = File::open(&path).unwrap_or_else(|_| {
      panic!(
        "Could not open configuration file specified by env var CONFIG_PATH={}",
        path
      )
    });
    ron::de::from_reader::<File, Config>(config)
  } else {
    let config = std::env::var("CONFIG")
      .unwrap_or_else(|_| panic!("Neither CONFIG_PATH nor CONFIG env vars were set."));

    ron::de::from_str::<Config>(&config)
  }
  .unwrap_or_else(|error| panic!("Could not parse configuration file: {}", error))
}

/// Returns the global configuration for the server.
pub fn server_config() -> &'static Config {
  CONFIG.get_or_init(init)
}
