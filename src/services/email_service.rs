use std::convert::{TryFrom, TryInto};

use anyhow::Result;
use rusoto_core::{self, Region};
use rusoto_ses::{
  BulkEmailDestination, Destination, SendBulkTemplatedEmailRequest, Ses, SesClient,
};
use serde_json::json;
use strum_macros::Display;

use crate::config;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailTemplate {
  pub template_type: EmailTemplateTypes,

  pub to_addresses: Vec<String>,

  pub params: Vec<EmailTemplateParams>,
}

#[derive(Serialize, Deserialize, Debug, Display, Clone)]
pub enum EmailTemplateTypes {
  #[strum(serialize = "inviteNewUserToProject")]
  InviteNewUserToProject,

  #[strum(serialize = "inviteNewUserToDashboard")]
  InviteNewUserToDashboard,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EmailTemplateParams {
  InviteNewUserToProject(InviteNewUserToProjectParams),
  InviteNewUserToDashboard(InviteNewUserToDashboardParams),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InviteNewUserToProjectParams {
  pub user: String,

  pub project_link: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InviteNewUserToDashboardParams {
  pub user: String,

  pub dashboard_link: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailContent {
  pub template_name: String,

  pub to_addresses: Vec<String>,

  // Contains list of replacement_data JSON strings for each destination email
  pub template_data: Vec<String>,
}

impl From<EmailTemplate> for EmailContent {
  fn from(template: EmailTemplate) -> Self {
    EmailContent {
      template_name: template.template_type.to_string(),
      to_addresses: template.to_addresses,
      template_data: template
        .params
        .iter()
        .map(|p| match p {
          EmailTemplateParams::InviteNewUserToProject(params) => json!(params).to_string(),
          EmailTemplateParams::InviteNewUserToDashboard(params) => json!(params).to_string(),
        })
        .collect(),
    }
  }
}

pub async fn send_email(template: EmailTemplate) -> Result<bool> {
  let config = config::server_config();
  let client = SesClient::new(Region::ApSoutheast1);
  let EmailContent {
    template_name,
    to_addresses,
    template_data,
  } = template.into();

  let destinations: Vec<BulkEmailDestination> = to_addresses
    .into_iter()
    .zip(template_data.into_iter())
    .map(|(address, data)| BulkEmailDestination {
      destination: Destination {
        bcc_addresses: None,
        cc_addresses: None,
        to_addresses: Some(vec![address]),
      },
      replacement_tags: None,
      replacement_template_data: Some(data),
    })
    .collect();

  let send_request = SendBulkTemplatedEmailRequest {
    destinations,
    // TODO: To change to definite server email source
    source: config.email_source.clone(),
    template: template_name,
    default_template_data: Some(json!({}).to_string()),
    ..Default::default()
  };

  client.send_bulk_templated_email(send_request).await?;
  println!("Email sent!");

  Ok(true)
}
