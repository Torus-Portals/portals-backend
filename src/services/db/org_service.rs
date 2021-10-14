use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{Executor, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBOrg {
  pub id: Uuid,

  pub name: String,

  pub personal: bool,

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
      personal: false,
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
  pub personal: bool,
}

// Needs to be scoped to the user's orgs...
pub async fn get_org<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  org_id: Uuid,
) -> Result<DBOrg> {
  sqlx::query_as!(DBOrg, "select * from orgs where id = $1", org_id)
    .fetch_one(pool)
    .await
    .map_err(anyhow::Error::from)
}

// Note: This is realllllllly bad, and will return all orgs in the db.
//       Should probably take an array of org ids.
pub async fn get_orgs<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  ids: &[Uuid],
) -> Result<Vec<DBOrg>> {
  sqlx::query_as!(DBOrg, "select * from orgs where id = any($1)", ids)
    .fetch_all(pool)
    .await
    .map_err(anyhow::Error::from)
}

pub async fn create_org<'e>(
  pool: impl Executor<'e, Database = Postgres>,
  auth0id: &str,
  new_org: DBNewOrg,
) -> Result<DBOrg> {
  sqlx::query_as!(
    DBOrg,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into orgs (name, personal, created_by, updated_by) values ($2, $3, (select id from _user), (select id from _user))
    returning id, name, personal, created_at, created_by, updated_at, updated_by
    "#,
    auth0id,
    new_org.name,
    new_org.personal
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

// pub async fn get_user_orgs(&self, user_id: Uuid) -> Result<Vec<DBOrg>> {
//   sqlx::query_as!(DBOrg,
//   r#"
//   with _user_orgs as (select orgs from users where id = $1)
//   select * from orgs where id = any _user_orgs
//   "#,
//   user_id
// ).fetch_all().await
// .map_err(anyhow::Error::from)
// }
