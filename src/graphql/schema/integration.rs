use chrono::{DateTime, Utc};
use juniper::{
  graphql_object, FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject,
  GraphQLUnion,
};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use crate::graphql::context::GQLContext;
use crate::services::db::integration_service::{
  create_integration, get_integration, get_integrations, DBIntegration, DBNewIntegration,
};
// use crate::services::google_sheets_service::fetch_sheets_value;

use super::{Mutation, Query};

#[derive(GraphQLEnum, EnumString, Display, Clone, Debug, Deserialize, Serialize)]
pub enum IntegrationTypes {
  #[strum(serialize = "GoogleSheets")]
  GoogleSheets,
}

// TODO: figure out how to pass this as GraphQLInput.
// Note that the spec says "Unions are never valid inputs" (https://spec.graphql.org/June2018/#sec-Unions)
// #[derive(GraphQLUnion, Clone, Debug, Deserialize, Serialize)]
// #[graphql(Context = GQLContext)]
// pub enum IntegrationData {
//   GoogleSheets(GoogleSheetsIntegration),
// }

#[derive(GraphQLObject, Clone, Debug, Deserialize, Serialize)]
#[graphql(Context = GQLContext)]
pub struct Integration {
  pub id: Uuid,

  pub name: String,

  pub portal_id: Uuid,

  pub integration_type: IntegrationTypes,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl From<DBIntegration> for Integration {
  fn from(db_integration: DBIntegration) -> Self {
    // let data = serde_json::from_value(db_integration.integration_data)
    //   .expect("Unable to deserialize JSON integration_data.");

    Integration {
      portal_id: db_integration.portal_id,
      id: db_integration.id,
      name: db_integration.name,
      integration_type: db_integration
        .integration_type
        .parse()
        .expect("Unable to convert integration_type string to enum variant"),
      created_at: db_integration.created_at,
      created_by: db_integration.created_by,
      updated_at: db_integration.updated_at,
      updated_by: db_integration.updated_by,
    }
  }
}

#[derive(GraphQLInputObject, Clone, Debug, Deserialize, Serialize)]
pub struct NewIntegration {
  pub portal_id: Uuid,

  pub name: String,

  pub integration_type: IntegrationTypes,
  // JSON response from API call
  // pub integration_data: GoogleSheetsIntegrationInput,
}

#[derive(GraphQLInputObject, Clone, Debug, Deserialize, Serialize)]
pub struct GoogleSheetsIntegrationInput {
  pub sheet_url: String,
  pub sheet_name: String,
}

#[derive(GraphQLObject, Clone, Debug, Deserialize, Serialize)]
pub struct GoogleSheetsIntegration {
  pub sheet_url: String,
  pub sheet_name: String,
  pub row_dimensions: Vec<String>,
  pub col_dimensions: Vec<String>,
}

impl Query {
  pub async fn integration_impl(
    ctx: &GQLContext,
    integration_id: Uuid,
  ) -> FieldResult<Integration> {
    get_integration(&ctx.pool, integration_id)
      .await
      .map(|db_integration| db_integration.into())
      .map_err(FieldError::from)
  }

  pub async fn integrations_impl(
    ctx: &GQLContext,
    portal_id: Uuid,
  ) -> FieldResult<Vec<Integration>> {
    get_integrations(&ctx.pool, portal_id)
      .await
      .map(|db_integrations| db_integrations.into_iter().map(|b| b.into()).collect())
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_integration_impl(
    ctx: &GQLContext,
    new_integration: NewIntegration,
  ) -> FieldResult<Integration> {
    // let sheets_obj = fetch_sheets_value(
    //   new_integration.integration_data.sheet_url.clone(),
    //   new_integration.integration_data.sheet_name.clone(),
    //   None,
    // )
    // .await;

    // let row_dimensions: Vec<String> = sheets_obj.value_ranges[0]
    //   .values
    //   .iter()
    //   .skip(1)
    //   .map(|row| row[0].clone())
    //   .collect();
    // let col_dimensions: Vec<String> = sheets_obj.value_ranges[0].values[0].clone();
    // let google_sheets_data = GoogleSheetsIntegration {
    //   sheet_url: new_integration.integration_data.sheet_url,
    //   sheet_name: new_integration.integration_data.sheet_name,
    //   row_dimensions: vec![],
    //   col_dimensions: vec![],
    // };

    let db_new_integration = DBNewIntegration {
      portal_id: new_integration.portal_id,
      name: new_integration.name,
      integration_type: new_integration.integration_type.to_string(),
      // integration_data: serde_json::to_value(google_sheets_data)
      //   .expect("Unable to serialize GoogleSheetsIntegration data into valid JSON format."),
    };

    create_integration(&ctx.pool, &ctx.auth0_user_id, db_new_integration)
      .await
      .map(|integration| integration.into())
      .map_err(FieldError::from)
  }

  // pub async fn delete_integration(ctx: &GQLContext, integration_id: Uuid) -> FieldResult<i32> {}
}
