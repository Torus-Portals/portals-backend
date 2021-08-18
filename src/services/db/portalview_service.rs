use super::DB;

use anyhow::Result;
use chrono::{DateTime, Utc};

use uuid::Uuid;

use crate::graphql::schema::portalview::{NewPortalView};

#[derive(Debug, Serialize)]
pub struct DBPortalView {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  pub egress: String,

  pub access: String,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

pub struct DBNewPortalView {
  pub portal_id: Uuid,

  pub name: String,

  pub egress: String,

  pub access: String,
}

impl From<NewPortalView> for DBNewPortalView {
  fn from(new_portalview: NewPortalView) -> Self {
    DBNewPortalView {
      portal_id: new_portalview.portal_id,
      name: new_portalview.name,
      egress: new_portalview.egress,
      access: new_portalview.access,
    }
  }
}

impl DB {
  pub async fn get_portal_views(&self, portal_id: Uuid) -> Result<Vec<DBPortalView>> {
    sqlx::query_as!(
      DBPortalView,
      r#"
      select * from portalviews where portal_id = $1
      "#,
      portal_id
    )
    .fetch_all(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }

  pub async fn create_portalview(
    &self,
    auth0_user_id: &str,
    new_portalview: DBNewPortalView,
  ) -> Result<DBPortalView> {
    sqlx::query_as!(
      DBPortalView,
      r#"
      with _user as (select * from users where auth0id = $1)
      insert into portalviews (
        portal_id,
        name,
        egress,
        access,
        created_by,
        updated_by
      ) values (
        $2,
        $3,
        $4,
        $5,
        (select id from _user),
        (select id from _user)
      ) returning *

      "#,
      auth0_user_id,
      new_portalview.portal_id,
      new_portalview.name,
      new_portalview.egress,
      new_portalview.access,
    )
    .fetch_one(&self.pool)
    .await
    .map_err(anyhow::Error::from)
  }
}
