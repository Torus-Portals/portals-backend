use actix_web::{get, Error, HttpResponse};
use once_cell::sync::OnceCell;

#[get("/health")]
pub async fn get_health() -> HttpResponse {
  HttpResponse::Ok().body(String::from("Hello from the other side!"))
}

#[allow(dead_code)]
mod info {
  include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Debug, Serialize)]
struct Info<'a> {
  app: &'a str,
  version: &'a str,
  target: &'a str,
  profile: &'a str,
  optimization_level: &'a str,
  git_head_ref: Option<&'a str>,
  git_commit_hash: Option<&'a str>,
  build_time_utc: &'a str,
}

static INFO: OnceCell<Info> = OnceCell::new();

#[get("/info")]
pub async fn get_info() -> Result<HttpResponse, Error> {
  let info = INFO.get_or_init(|| Info {
    app: info::PKG_NAME,
    version: info::PKG_VERSION,
    target: info::TARGET,
    profile: info::PROFILE,
    optimization_level: info::OPT_LEVEL,
    git_head_ref: info::GIT_HEAD_REF,
    git_commit_hash: info::GIT_COMMIT_HASH,
    build_time_utc: info::BUILT_TIME_UTC,
  });
  Ok(HttpResponse::Ok().json(info))
}
