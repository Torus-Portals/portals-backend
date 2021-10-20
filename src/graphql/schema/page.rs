use chrono::{DateTime, Utc};
use juniper::{graphql_object, FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use uuid::Uuid;

use crate::services::db::page_service::DBPage;


#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct Page {
  pub id: Uuid,

  pub name: String,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

impl From<DBPage> for Page {
  fn from(page: DBPage) -> Self {
    Page {
      id: page.id,
      name: page.name,
      created_at: page.created_at,
      created_by: page.created_by,
      updated_at: page.updated_at,
      updated_by: page.updated_by,
    }
  }
}
