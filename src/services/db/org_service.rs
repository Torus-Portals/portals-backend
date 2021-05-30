use super::DB;
// use crate::models::db_org::{DBOrg, NewDBOrg};
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use uuid::Uuid;
#[derive(Debug, Serialize, Deserialize)]
pub struct DBOrg {
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

impl Default for DBOrg {
  fn default() -> Self {
    DBOrg {
      id: Uuid::new_v4(),
      name: "not_a_real_org".to_string(),
      created_at: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
      created_by: Uuid::new_v4(),
      updated_at: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
      updated_by: Uuid::new_v4(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBNewOrg {
  pub name: String,
}

impl DB {
  // Needs to be scoped to the user's orgs...
  pub async fn get_org(&self, org_id: Uuid) -> Result<DBOrg> {
    sqlx::query_as!(DBOrg, "select * from orgs where id = $1", org_id)
      .fetch_one(&self.pool)
      .await
      .map_err(anyhow::Error::from)
  }

  pub async fn get_orgs(&self) -> Result<Vec<DBOrg>> {
    sqlx::query_as!(DBOrg, "select * from orgs")
      .fetch_all(&self.pool)
      .await
      .map_err(anyhow::Error::from)
  }

  pub async fn create_org(&self, auth0id: &str, new_org: DBNewOrg) -> Result<DBOrg> {
    sqlx::query_as!(
      DBOrg,
      r#"
      with _user as (select * from users where auth0id = $1)
      insert into orgs (name, created_by, updated_by) values ($2, (select id from _user), (select id from _user))
      returning name, id, created_at, created_by, updated_at, updated_by
      "#,
      auth0id,
      new_org.name
    )
    .fetch_one(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }
}
