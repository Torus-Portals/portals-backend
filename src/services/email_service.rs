use std::convert::{TryFrom, TryInto};

use rusoto_core::{self, Region};
use rusoto_ses::{
  BulkEmailDestination, Destination, SendBulkTemplatedEmailRequest, Ses, SesClient,
};
use serde_json::json;
use strum_macros::Display;

#[derive(Serialize, Deserialize, Debug, Display, Clone)]
pub enum EmailTemplate {
  #[strum(serialize = "inviteNewUserToPortal")]
  InviteNewUserToPortal(Vec<InviteNewUserToPortalParams>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

pub async fn send_email(template: EmailTemplate) -> Result<bool, anyhow::Error> {
  let client = SesClient::new(Region::ApSoutheast1);
  let EmailContent {
    template_name,
    to_addresses,
    template_data,
  } = template.try_into()?;

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
    source: "tedmundhtl@gmail.com".to_string(),
    template: template_name,
    default_template_data: Some(json!({}).to_string()),
    ..Default::default()
  };

  client.send_bulk_templated_email(send_request).await?;
  println!("Email sent!");

  Ok(true)
}
