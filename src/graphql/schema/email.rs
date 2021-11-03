use std::convert::{TryFrom, TryInto};

use juniper::{FieldError, FieldResult, GraphQLInputObject};

use crate::graphql::context::GQLContext;
use crate::graphql::schema::Mutation;

use serde_json::json;
use strum_macros::Display;

#[derive(Serialize, Deserialize, Debug, Display, Clone)]
pub enum EmailTemplate {
  #[strum(serialize = "inviteNewUserToPortal")]
  InviteNewUserToPortal(Vec<InviteNewUserToPortalParams>),
}

#[derive(GraphQLInputObject, Serialize, Deserialize, Debug, Clone)]
pub struct InviteNewUserToPortalParams {
  // Skip the `to_address` field when serializing
  // This allows us to pass the JSON string to `replacement_template_data` in `BulkEmailDestination` directly
  #[serde(skip)]
  to_address: String,

  user: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailContent {
  pub template_name: String,

  pub to_addresses: Vec<String>,

  // Contains list of replacement_data JSON strings for each destination email
  pub template_data: Vec<String>,
}

impl TryFrom<EmailTemplate> for EmailContent {
  type Error = anyhow::Error;

  fn try_from(template: EmailTemplate) -> Result<Self, Self::Error> {
    let params = match &template {
      EmailTemplate::InviteNewUserToPortal(params) => params,
    };

    Ok(EmailContent {
      template_name: template.to_string(),
      to_addresses: params.iter().map(|p| p.to_address.clone()).collect(),
      template_data: params.iter().map(|p| json!(p).to_string()).collect(),
    })
  }
}

impl Mutation {
  pub async fn send_invite_new_user_to_portal_email_impl(
    ctx: &GQLContext,
    params: Vec<InviteNewUserToPortalParams>,
  ) -> FieldResult<bool> {
    let email = ctx.email.lock().await;

    let email_content = EmailTemplate::InviteNewUserToPortal(params).try_into()?;

    email
      .send_email(email_content)
      .await
      .map_err(FieldError::from)
  }
}
