use juniper::{FieldResult, GraphQLInputObject, GraphQLObject};
use uuid::Uuid;

use crate::graphql::context::GQLContext;
use crate::config;
use crate::graphql::schema::integration::{IntegrationTypes, NewIntegration};
use crate::graphql::schema::{Query, Mutation};
use crate::services::db::integration_service::create_integration;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct GoogleSheetsRedirectURI {
  pub redirect_uri: String,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct GoogleSheetsAuthorization {
  pub code: String,
}

impl Query {
  pub async fn google_sheets_redirect_uri_impl(state: String) -> FieldResult<GoogleSheetsRedirectURI> {
    let config = config::server_config();

    let redirect_uri = format!(
      "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&state={}",
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
      state
    );

    Ok(GoogleSheetsRedirectURI { redirect_uri })
  }
}

impl Mutation {
  pub async fn authorize_google_sheets_impl(
    ctx: &GQLContext, 
    portal_id: Uuid, 
    auth: GoogleSheetsAuthorization
  ) -> FieldResult<bool> {
    let mut gs = ctx.google_sheets.lock().await;

    // This call needs to store the tokens that are retrieved. 
    let gsheets_token = gs.exchange_code(auth.code).await?;
    // Integration should be created only after access_token successfully exchanged
    let new_integration = NewIntegration {
      portal_id,
      // Sensible name -- email + hash?
      name: "IntegrationTest".to_string(),
      integration_type: IntegrationTypes::GoogleSheets,
    };
    let integration = create_integration(
      &ctx.pool, 
      ctx.auth0_user_id.as_str(),
      new_integration.into()
    ).await?;
    
    Ok(gs.store_token(integration.id, gsheets_token).await?)
  }
}
