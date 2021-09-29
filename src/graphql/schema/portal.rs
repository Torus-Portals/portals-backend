use chrono::{DateTime, Utc};
use juniper::{graphql_object, FieldError, FieldResult, GraphQLInputObject};

use uuid::Uuid;

use super::Mutation;
use super::Query;

use crate::graphql::context::GQLContext;
use crate::graphql::schema::user::{User, NewUser};
use crate::services::db::portal_service::*;
use crate::services::db::portalview_service::{create_portalview, DBNewPortalView};
use crate::services::db::user_service::{
  create_user_with_new_org, get_user_by_email, user_exists_by_email,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Portal {
  pub id: Uuid,

  pub name: String,

  pub org_id: Uuid,

  pub owner_ids: Vec<Uuid>,

  pub vendor_ids: Vec<Uuid>,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

#[graphql_object(context = GQLContext)]
impl Portal {
  fn id(&self) -> Uuid {
    self.id
  }

  fn name(&self) -> String {
    self.name.clone()
  }

  fn org_id(&self) -> Uuid {
    self.org_id
  }

  fn owner_ids(&self) -> Vec<Uuid> {
    self
      .owner_ids
      .clone()
  }

  fn vendor_ids(&self) -> Vec<Uuid> {
    self
      .vendor_ids
      .clone()
  }

  fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  fn created_by(&self) -> Uuid {
    self.created_by
  }

  fn updated_at(&self) -> DateTime<Utc> {
    self.updated_at
  }

  fn updated_by(&self) -> Uuid {
    self.updated_by
  }
}

impl From<DBPortal> for Portal {
  fn from(db_portal: DBPortal) -> Self {
    Portal {
      id: db_portal.id,
      name: db_portal.name,
      org_id: db_portal.org_id,
      owner_ids: db_portal.owner_ids,
      vendor_ids: db_portal.vendor_ids,
      created_at: db_portal.created_at,
      created_by: db_portal.created_by,
      updated_at: db_portal.updated_at,
      updated_by: db_portal.updated_by,
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewPortal {
  pub name: String,

  pub org_id: Uuid,

  pub owner_ids: Vec<Uuid>,

  pub vendor_ids: Vec<Uuid>,
}

impl From<DBNewPortal> for NewPortal {
  fn from(db_new_portal: DBNewPortal) -> Self {
    NewPortal {
      org_id: db_new_portal.org_id,
      name: db_new_portal.name,
      owner_ids: db_new_portal.owner_ids,
      vendor_ids: db_new_portal.vendor_ids,
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct UpdatePortal {
  pub id: Uuid,

  pub name: Option<String>,

  pub owner_ids: Option<Vec<Uuid>>,

  pub vendor_ids: Option<Vec<Uuid>>,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct PortalInviteParams {
  pub portal_id: Uuid,

  pub user_email: String,

  pub egress: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortalAndUsers {
  pub portal: Portal,
  
  pub users: Vec<User>,
}

#[graphql_object(context = GQLContext)]
impl PortalAndUsers {
  fn portal(&self) -> Portal {
    self.portal.clone()
  }

  fn users(&self) -> Vec<User> {
    self.users.clone()
  }
}

impl Query {
  pub async fn portal_impl(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Portal> {
    get_portal(&ctx.pool, portal_id)
      .await
      .map(|db_portal| db_portal.into())
      .map_err(FieldError::from)
  }

  // Get all portals associated to a user
  pub async fn user_portals_impl(ctx: &GQLContext) -> FieldResult<Vec<Portal>> {
    get_auth0_user_portals(&ctx.pool, &ctx.auth0_user_id)
      .await
      .map(|db_portals| {
        db_portals
          .into_iter()
          .map(|p| p.into())
          .collect()
      })
      .map_err(FieldError::from)
  }

  pub async fn portals_by_ids_impl(
    ctx: &GQLContext,
    portal_ids: Vec<Uuid>,
  ) -> FieldResult<Vec<Portal>> {
    get_portals(&ctx.pool, portal_ids)
      .await
      .map(|db_portals| {
        db_portals
          .into_iter()
          .map(|p| p.into())
          .collect()
      })
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_portal_impl(ctx: &GQLContext, new_portal: NewPortal) -> FieldResult<Portal> {
    let portal: Portal = create_portal(&ctx.pool, &ctx.auth0_user_id, new_portal.into())
      .await
      .map(|db_portal| db_portal.into())
      .map_err(FieldError::from)?;

    // Create a default owner and vendor portalview
    create_portalview(
      &ctx.pool,
      &ctx.auth0_user_id,
      DBNewPortalView {
        portal_id: portal.id,
        name: String::from("Default Owner View"),
        egress: String::from("owner"),
        access: String::from("private"),
      },
    )
    .await?;

    create_portalview(
      &ctx.pool,
      &ctx.auth0_user_id,
      DBNewPortalView {
        portal_id: portal.id,
        name: String::from("Default Vendor View"),
        egress: String::from("vendor"),
        access: String::from("private"),
      },
    )
    .await?;

    Ok(portal)
  }

  pub async fn update_portal_impl(
    ctx: &GQLContext,
    portal_update: UpdatePortal,
  ) -> FieldResult<Portal> {
    update_portal(&ctx.pool, &ctx.auth0_user_id, portal_update.into())
      .await
      .map(|db_portal| db_portal.into())
      .map_err(FieldError::from)
  }

  pub async fn delete_portal_impl(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<i32> {
    let local_pool = ctx.pool.clone();
    delete_portal(local_pool, portal_id)
      .await
      .map_err(FieldError::from)
  }

  pub async fn invite_user_to_portal_impl(
    ctx: &GQLContext,
    portal_invite_params: PortalInviteParams,
  ) -> FieldResult<PortalAndUsers> {
    // Check to see if user already exists in Portals
    // let local_pool = ctx.pool.clone();

    let portal = get_portal(&ctx.pool, portal_invite_params.portal_id).await?;

    let exists = user_exists_by_email(&ctx.pool, &portal_invite_params.user_email).await?;

    let user = match exists {
      true => {
        get_user_by_email(&ctx.pool, &portal_invite_params.user_email).await?
      }
      false => {
        let new_user = NewUser {
          name: String::new(),
          nickname: String::new(),
          email: portal_invite_params
            .user_email
            .to_owned(),
          status: String::from("invited"),
          org_ids: None,
          role_ids: None,
        };

        let user_and_org = create_user_with_new_org(
          ctx.pool.clone(),
          &ctx.auth0_user_id,
          new_user.into_db_new_user(String::new()),
        )
        .await?;

        user_and_org.0
      }
    };

    // update portal
    let portal_update = match portal_invite_params.egress.as_str() {
      "owner" => {
        let mut owner_ids = portal.owner_ids;

        owner_ids.push(user.id);

        Ok(DBUpdatePortal {
            id: portal.id,
            name: None,
            owner_ids: Some(owner_ids),
            vendor_ids: None,
        })
      },
      "vendor" => {
        let mut vendor_ids = portal.vendor_ids;

        vendor_ids.push(user.id);

        Ok(DBUpdatePortal {
            id: portal.id,
            name: None,
            owner_ids: None,
            vendor_ids: Some(vendor_ids),
        })
      },
      _ => Err("portal_invite_params.egress is neither owner or vendor")
    }?;

    let updated_portal = update_portal(&ctx.pool, &ctx.auth0_user_id, portal_update).await?;

    Ok(PortalAndUsers {
      portal: updated_portal.into(),
      users: vec![user.into()],
    })
  }
}
