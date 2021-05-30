// use actix_web::{dev, error, FromRequest, HttpRequest};
// use chrono::{DateTime, Utc};
// use serde_json;
// use uuid::Uuid;

// #[derive(Debug, Serialize, Deserialize)]
// pub struct DBBlock {
//   pub id: Uuid,

//   #[serde(rename = "blockType")]
//   pub block_type: String,

//   #[serde(rename = "portalId")]
//   pub portal_id: Uuid,

//   #[serde(rename = "portalViewId")]
//   pub portal_view_id: Uuid,

//   pub egress: String,

//   pub bbox: Vec<i32>,

//   pub data: serde_json::Value,

//   #[serde(rename = "createdAt")]
//   pub created_at: DateTime<Utc>,

//   #[serde(rename = "createdBy")]
//   pub created_by: Uuid,

//   #[serde(rename = "updatedAt")]
//   pub updated_at: DateTime<Utc>,

//   #[serde(rename = "updatedBy")]
//   pub updated_by: Uuid,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct NewBlock {
//   pub block_type: String,
//   pub portal_id: Uuid,
//   pub portal_view_id: Uuid,
//   pub egress: String,
//   pub bbox: Vec<i32>,
//   pub data: serde_json::Value,
//   #[serde(rename = "createdBy")]
//   pub created_by: Uuid,
//   #[serde(rename = "updatedBy")]
//   pub updated_by: Uuid,
// }

// #[derive(Serialize, Deserialize, JSONPayload)]
// pub struct NewBlockPayload {
//   #[serde(rename = "blockType")]
//   pub block_type: String,

//   #[serde(rename = "portalId")]
//   pub portal_id: Uuid,

//   #[serde(rename = "portalViewId")]
//   pub portal_view_id: Uuid,

//   pub egress: String,

//   pub bbox: Vec<i32>,

//   pub data: serde_json::Value,
// }
