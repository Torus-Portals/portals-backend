use std::sync::Arc;

use crate::services::auth0_service::Auth0Service;
use crate::state::State;
use actix_web::{dev, web, Error, HttpResponse};

use actix_web_httpauth::middleware::HttpAuthentication;
use futures::lock::Mutex;

use super::context::GQLContext;
use super::juniper_actix::{graphql_handler, playground_handler};
use super::schema::Schema;
use crate::middleware::auth::validator;
use crate::models::user::Auth0UserId;

// async fn get_decoded_token()

// GraphQL

async fn graphql_route(
  req: actix_web::HttpRequest,
  payload: actix_web::web::Payload,
  schema: web::Data<Schema>,
  state: web::Data<State>,
  auth0_api: web::Data<Arc<Mutex<Auth0Service>>>,
  auth0_user_id: Auth0UserId,
) -> Result<HttpResponse, Error> {
  let p = state.pool.clone();
  let a = auth0_api.into_inner();

  let ctx = GQLContext::new(p, auth0_user_id.id, a);

  graphql_handler(schema.get_ref(), &ctx, req, payload).await
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
