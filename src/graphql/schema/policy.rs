use chrono::{DateTime, Utc};
use juniper::{GraphQLEnum, GraphQLObject};
use strum_macros::{Display, EnumIter, EnumString};
use uuid::Uuid;

use crate::graphql::context::GQLContext;

#[derive(Debug, Serialize, Deserialize, EnumString, Display)]
pub enum PolicyTypes {
  #[strum(serialize = "Project")]
  ProjectPolicy,

  #[strum(serialize = "Dashboard")]
  DashboardPolicy,

  #[strum(serialize = "Page")]
  PagePolicy,

  #[strum(serialize = "Block")]
  BlockPolicy,
}

#[derive(Debug, Serialize, Deserialize, EnumString, Display)]
pub enum PermissionTypes {
  #[strum(serialize = "Dashboard")]
  DashboardPermission,

  #[strum(serialize = "Page")]
  PagePermission,

  #[strum(serialize = "Block")]
  BlockPermission,
}

#[derive(Debug, Serialize, Deserialize, EnumString, EnumIter, Display)]
pub enum GrantTypes {
  All,
  Create,
  Read,
  Update,
  Delete,
}

#[derive(Debug, Serialize, Deserialize)]
// #[graphql(Context = GQLContext)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
  pub id: Uuid,

  pub resource_id: Uuid,

  pub policy_type: PolicyTypes,

  pub permission_type: PermissionTypes,

  pub grant_type: GrantTypes,

  pub user_ids: Vec<Uuid>,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

// TODO: Created implicitly? With certain operations
// e.g. if user creates a project, the user should have full
// ownership of the project (all grants, etc) -> would create 4 policies,
// one for each grant.
// But may have certain operations that create only one grant for a particular user
// who is not the owner, naturally.
#[derive(Debug, Serialize, Deserialize)]
pub struct NewPolicy {
  pub resource_id: Uuid,

  pub policy_type: PolicyTypes,

  pub permission_type: PermissionTypes,

  pub grant_type: GrantTypes,

  pub user_ids: Vec<Uuid>,
}

pub struct UpdatePolicy {
  pub id: Uuid,

  pub policy_type: PolicyTypes,

  pub permission_type: PermissionTypes,

  pub grant_type: GrantTypes,

  pub user_ids: Vec<Uuid>,
}
