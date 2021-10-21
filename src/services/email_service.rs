use futures::lock::Mutex;
use std::{
  convert::{TryFrom, TryInto},
  sync::Arc,
};

use actix_web::{dev, web, Error, HttpResponse};
use rusoto_core::{self, Region};
use rusoto_ses::{
  BulkEmailDestination, Destination, SendBulkTemplatedEmailRequest, Ses, SesClient,
};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailData {
  pub template: EmailTemplate,
  pub to_addresses: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
// Using adjacently-tagged representation
// A JSON string for this enum representation may look something like:
// { "type": "InviteNewUserToPortal", "data": [{...}, {...}] }
// See https://serde.rs/enum-representations.html for more information.
pub enum EmailTemplate {
  InviteNewUserToPortal(Vec<InviteNewUserToPortalParams>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InviteNewUserToPortalParams {
  user: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailContent {
  pub template_name: String,
  // Contains list of replacement_data JSON strings for each destination email
  pub template_data: Vec<String>,
}

impl TryFrom<EmailTemplate> for EmailContent {
  type Error = anyhow::Error;

  fn try_from(template: EmailTemplate) -> Result<Self, Self::Error> {
    let content = match template {
      EmailTemplate::InviteNewUserToPortal(params) => EmailContent {
        template_name: "inviteNewUserToPortal".to_string(),
        template_data: params.into_iter().map(|p| json!(p).to_string()).collect(),
      },
    };

    Ok(content)
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailService {}

impl EmailService {
  pub fn new() -> Self {
    EmailService {}
  }

  pub async fn send_email(
    &self,
    email_template: EmailTemplate,
    to_addresses: Vec<String>,
  ) -> Result<bool, anyhow::Error> {
    let client = SesClient::new(Region::ApSoutheast1);
    let EmailContent {
      template_name,
      template_data,
    } = email_template.try_into()?;

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
}

// #[post("/email")]
pub async fn post_invitation_email(
  email: web::Data<Arc<Mutex<EmailService>>>,
  data: web::Json<EmailData>,
) -> Result<HttpResponse, Error> {
  dbg!(&data);
  let email_service = email.lock().await;
  let data = data.into_inner();
  email_service
    .send_email(data.template, data.to_addresses)
    .await
    .expect("Unable to send email!");
  Ok(HttpResponse::Ok().body("Invite successfully sent!"))
}

pub fn get_email_routes() -> impl dev::HttpServiceFactory + 'static {
  web::resource("/email").route(web::post().to(post_invitation_email))
}
