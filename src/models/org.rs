use chrono::{DateTime, NaiveDateTime, Utc};
use juniper::{GraphQLObject, GraphQLInputObject};
use uuid::Uuid;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct Org {
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

impl Default for Org {
  fn default() -> Self {
    Org {
      id: Uuid::new_v4(),
      name: "not_a_real_org".to_string(),
      created_at: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
      created_by: Uuid::new_v4(),
      updated_at: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
      updated_by: Uuid::new_v4(),
    }
  }
}

// #[derive(Serialize, Deserialize, JSONPayload)]
#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewOrg {
  pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct IsertableNewOrg {
  pub name: String,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}
