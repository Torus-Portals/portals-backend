// use crate::utils::async_block::{ BlockError };
// use serde_json::json;

// use rusoto_core::Region;
// use rusoto_ses::{ Ses, SesClient, ListTemplatesRequest, ListTemplatesResponse, SendTemplatedEmailRequest, Destination, Message };

// // Docs: https://docs.rs/rusoto_ses/0.43.0/rusoto_ses/#structs

// pub async fn send_invitation_email() -> Result<(), BlockError> {
//   println!("In send_invitation_email");

//   let client = SesClient::new(Region::UsWest2);

//   let req = ListTemplatesRequest {
//     max_items: Some(100),
//     next_token: None,
//   };

//   let result: ListTemplatesResponse = client.list_templates(req).await.unwrap();

//   println!("email result: {:?}", result);
//   let result = client.list_verified_email_addresses().await.unwrap();

  
//   let tmpl_data = json!({
//     "user": "Another Person",
//   });

//   let a = SendTemplatedEmailRequest {
//     destination: Destination {
//     bcc_addresses: None,
//     cc_addresses: None,
//     to_addresses: Some(vec![String::from("sinexo2900@psk3n.com")]),
//     },
//     source: String::from("broch@torus.rocks"),
//     template: String::from("inviteNewUserToPortal"),
//     template_data: tmpl_data.to_string(),
//     ..Default::default()
//   };

//   match client.send_templated_email(a).await {
//     Ok(_) => println!("email was sent!!!"),
//     Err(err) => println!("email wasn't sent....: {:?}", err),
//   };
    

//   // println!("sent response???: {:?}", r);

//   println!("verified emails??: {:?}", result);
//   Ok(())
// }
