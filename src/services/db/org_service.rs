use super::DB;
use crate::models::org::{Org, NewOrg};
use anyhow::Result;
use uuid::Uuid;

impl DB {
  pub async fn get_org(&self, org_id: Uuid) -> Result<Org> {
    sqlx::query_as!(Org, "select * from orgs where id = $1", org_id)
      .fetch_one(&self.pool)
      .await
      .map_err(anyhow::Error::from)
  }

  pub async fn get_orgs(&self) -> Result<Vec<Org>> {
    sqlx::query_as!(Org, "select * from orgs")
      .fetch_all(&self.pool)
      .await
      .map_err(anyhow::Error::from)
  }

  pub async fn create_org(&self, auth0id: &str, new_org: NewOrg) -> Result<Org> {
    sqlx::query_as!(
      Org,
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
