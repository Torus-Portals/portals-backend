use juniper::{FieldResult, GraphQLInputObject, GraphQLObject};

use crate::graphql::context::GQLContext;
use crate::config;
use crate::graphql::schema::{Query, Mutation};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct GoogleSheetsRedirectURI {
  pub redirect_uri: String,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct GoogleSheetsAuthorization {
  pub code: String,
}

impl Query {
  pub async fn google_sheets_redirect_uri_impl() -> FieldResult<GoogleSheetsRedirectURI> {
    let config = config::server_config();

    let redirect_uri = format!(
      "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline",
      config
        .oauth
        .auth_url,
      config
        .oauth
        .client_id,
      config
        .oauth
        .auth_redirect_url,
      config.oauth.scope,
    );

    Ok(GoogleSheetsRedirectURI { redirect_uri })
  }
}

impl Mutation {
  pub async fn authorize_google_sheets_impl(ctx: &GQLContext, auth: GoogleSheetsAuthorization) -> FieldResult<bool> {
    let mut gs = ctx.google_sheets.lock().await;

    // This call needs to store the tokens that are retrieved. 
    let _access_token = gs.exchange_code(auth.code).await?;

    Ok(true)
  }
}
