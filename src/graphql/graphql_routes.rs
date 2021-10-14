use std::sync::Arc;

use crate::services::auth0_service::Auth0Service;
use crate::services::google_sheets_service::GoogleSheetsService;
use crate::state::State;
use actix_web::{dev, web, Error, HttpResponse};

use actix_web_httpauth::middleware::HttpAuthentication;
use futures::lock::Mutex;

use super::context::GQLContext;
use juniper_actix::{graphql_handler, playground_handler};
use super::schema::Schema;
use crate::middleware::auth::validator;
// use crate::models::user::Auth0UserId;
use crate::extractors::auth0_user_id::{Auth0UserId};

// async fn get_decoded_token()

// GraphQL

async fn graphql_route(
  req: actix_web::HttpRequest,
  payload: actix_web::web::Payload,
  schema: web::Data<Schema>,
  state: web::Data<State>,
  auth0_api: web::Data<Arc<Mutex<Auth0Service>>>,
  google_sheets: web::Data<Arc<Mutex<GoogleSheetsService>>>,
  auth0_user_id: Auth0UserId,
) -> Result<HttpResponse, Error> {
  let p = state.pool.clone();
  let s = schema.get_ref();
  let a = auth0_api.into_inner();
  let gs = google_sheets.into_inner();
  let ctx = GQLContext::new(p, auth0_user_id.id, a, gs);

  graphql_handler(s, &ctx, req, payload).await
}

pub fn get_graphql_routes() -> impl dev::HttpServiceFactory + 'static {
  web::resource("/graphql")
    .wrap(HttpAuthentication::bearer(validator))
    .route(web::get().to(graphql_route))
    .route(web::post().to(graphql_route))
}

// GraphQL Playground

pub async fn playground_route() -> Result<HttpResponse, Error> {
  playground_handler("/graphql", None).await
}

pub fn get_graphql_dev_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/dev").route("/playground", web::get().to(playground_route))
}
