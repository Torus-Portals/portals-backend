use chrono::{DateTime, Utc};
use juniper::{
  FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, GraphQLUnion,
};
use std::str::FromStr;
use strum_macros::{EnumString, ToString};
use uuid::Uuid;

use crate::graphql::context::GQLContext;
use crate::services::db::role_service::{DBNewRole, DBRole, get_role, create_role};

use super::Query;
use super::Mutation;

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, ToString)]
pub enum RoleTypes {
  System,
  Org,
  Portal,
}

// Perms

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct SystemPermissions {
  create_org: bool,
  view_org: bool,
  edit_org: bool,
  delete_org: bool,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct OrgPermissions {
  create_portal: bool,
  view_all_portals: bool,
  view_member_portals: bool,
  edit_org: bool,
  delete_org: bool,

  // Manipulate users within a portal
  add_org_user: bool,
  delete_org_user: bool,
  edit_org_users: bool,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct PortalPermissions {
  view_portal: bool,
  edit_portal: bool,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct EmptyPermissions {
  role_type: String,
}

#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
pub enum RolePerms {
  System(SystemPermissions),
  Org(OrgPermissions),
  Portal(PortalPermissions),
  Empty(EmptyPermissions),
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct Role {
  pub id: Uuid,

  pub role_type: RoleTypes,

  pub perms: RolePerms,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

impl From<DBRole> for Role {
  fn from(db_role: DBRole) -> Self {
    let perms = match db_role
      .role_type
      .as_str()
    {
      "System" => {
        let json: SystemPermissions =
          serde_json::from_value(db_role.perms).expect("Unable to deserialize SystemPermissions");
        RolePerms::System(json)
      }
      "Org" => {
        let json: OrgPermissions =
          serde_json::from_value(db_role.perms).expect("Unable to deserialize OrgPermissions");
        RolePerms::Org(json)
      }
      "Portal" => {
        let json: PortalPermissions =
          serde_json::from_value(db_role.perms).expect("Unable to deserialize PortalPermissions");
        RolePerms::Portal(json)
      }
      &_ => RolePerms::Empty(EmptyPermissions {
        role_type: String::from("Empty"),
      }),
    };

    let role_type = RoleTypes::from_str(
      db_role
        .role_type
        .as_str(),
    )
    .expect("Unable to convert role_type string to enum variant");

    Role {
      id: db_role.id,
      role_type,
      perms,
      created_at: db_role.created_at,
      created_by: db_role.created_by,
      updated_at: db_role.updated_at,
      updated_by: db_role.updated_by,
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewRole {
  pub role_type: RoleTypes,
  // pub perms: serde_json::Value,
}

impl From<DBNewRole> for NewRole {
  fn from(db_new_role: DBNewRole) -> Self {
    let role_type = RoleTypes::from_str(
      db_new_role
        .role_type
        .as_str(),
    )
    .expect("Unable to convert role_type string to enum variant");

    NewRole { role_type }
  }
}

impl Query {
  pub async fn role_impl(ctx: &GQLContext, role_id: Uuid) -> FieldResult<Role> {
      get_role(&ctx.pool, role_id)
      .await
      .map(|db_role| db_role.into())
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_role_impl(ctx: &GQLContext, new_role: NewRole) -> FieldResult<Role> {
      create_role(&ctx.pool, &ctx.auth0_user_id, new_role.into())
      .await
      .map(|role| -> Role { role.into() })
      .map_err(FieldError::from)
  }
}
