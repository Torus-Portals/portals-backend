use juniper::{FieldResult, GraphQLInputObject, GraphQLObject};
use uuid::Uuid;

use crate::config;
use crate::graphql::context::GQLContext;
use crate::graphql::schema::integration::{IntegrationTypes, NewIntegration};
use crate::graphql::schema::{Mutation, Query};
use crate::services::db::integration_service::create_integration;
use crate::services::google_sheets_service::{GoogleSheetsSheetDimensions, GoogleSheetsSpreadsheet, GoogleSheetsToken};

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
      "{}?client_id={}&redirect_uri={}&response_type=code&scope={}+{}&prompt=consent&access_type=offline&state={}",
      config
        .oauth
        .auth_url,
      config
        .oauth
        .client_id,
      config
        .oauth
        .auth_redirect_url,
      config.oauth.sheets_scope,
      config.oauth.drive_scope,
      state
    );

    Ok(GoogleSheetsRedirectURI { redirect_uri })
  }

  pub async fn google_sheets_list_spreadsheets_impl(
    ctx: &GQLContext,
    integration_id: Uuid,
  ) -> FieldResult<Vec<GoogleSheetsSpreadsheet>> {
    let gs = ctx.google_sheets.lock().await;

    Ok(gs.list_spreadsheets(integration_id).await?)
  }

  pub async fn google_sheets_list_spreadsheets_sheets_names_impl(
    ctx: &GQLContext,
    integration_id: Uuid,
    spreadsheet_id: String,
  ) -> FieldResult<Vec<String>> {
    let gs = ctx.google_sheets.lock().await;

    Ok(
      gs.list_spreadsheet_sheets_names(integration_id, spreadsheet_id)
        .await?,
    )
  }

  pub async fn google_sheets_fetch_sheet_dimensions_impl(
    ctx: &GQLContext,
    integration_id: Uuid,
    spreadsheet_id: String,
    sheet_name: String,
  ) -> FieldResult<GoogleSheetsSheetDimensions> {
    let gs = ctx.google_sheets.lock().await;

    Ok(gs.fetch_sheet_dimensions(integration_id, spreadsheet_id, sheet_name).await?)
  }
  
  // Needed? Or should only be handled by googlesheetscell (tho not likely as a embedded resolver)
  pub async fn google_sheets_fetch_sheet_values_impl(
    ctx: &GQLContext,
    integration_id: Uuid,
    spreadsheet_id: String,
    sheet_name: String,
    range: String,
  ) -> FieldResult<String> {
    let gs = ctx.google_sheets.lock().await;
    let sheet_range = format!("{}!{}", sheet_name, range);

    Ok(gs.fetch_sheet_value(integration_id, spreadsheet_id, sheet_range).await?)
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
