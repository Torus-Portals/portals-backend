use rusoto_core::{self, Region};
use rusoto_ses::{
  BulkEmailDestination, Destination, SendBulkTemplatedEmailRequest, Ses, SesClient,
};
use serde_json::json;

use crate::graphql::schema::email::EmailContent;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailService {}

impl EmailService {
  pub fn new() -> Self {
    EmailService {}
  }

  pub async fn send_email(&self, email_content: EmailContent) -> Result<bool, anyhow::Error> {
    let client = SesClient::new(Region::ApSoutheast1);
    let EmailContent {
      template_name,
      to_addresses,
      template_data,
    } = email_content;

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
