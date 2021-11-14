use std::{
  convert::{TryFrom, TryInto},
  str::FromStr,
};

use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject};
use strum_macros::{Display, EnumIter, EnumString};
use uuid::Uuid;

use crate::{
  graphql::context::GQLContext,
  services::db::policy_service::{update_policy, DBPolicy},
};

use super::Mutation;

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
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

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum PermissionTypes {
  #[strum(serialize = "Dashboard")]
  DashboardPermission,

  #[strum(serialize = "Page")]
  PagePermission,

  #[strum(serialize = "Block")]
  BlockPermission,
}

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, EnumIter, Display)]
pub enum GrantTypes {
  All,
  Create,
  Read,
  Update,
  Delete,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
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

impl TryFrom<DBPolicy> for Policy {
  type Error = anyhow::Error;

  fn try_from(db_policy: DBPolicy) -> Result<Self, Self::Error> {
    Ok(Policy {
      id: db_policy.id,
      resource_id: db_policy.resource_id,
      policy_type: PolicyTypes::from_str(&db_policy.policy_type)?,
      permission_type: PermissionTypes::from_str(&db_policy.permission_type)?,
      grant_type: GrantTypes::from_str(&db_policy.grant_type)?,
      user_ids: db_policy.user_ids,
      created_at: db_policy.created_at,
      created_by: db_policy.created_by,
      updated_at: db_policy.updated_at,
      updated_by: db_policy.updated_by,
    })
  }
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

// Used primarily through the GQL interface to update the policy for an existing
// resource (Project/Dashboard/Page/etc).
// Entails granting specific access to a user to a particular resource.
#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct UpdatePolicy {
  pub id: Uuid,

  pub policy_type: PolicyTypes,

  pub permission_type: PermissionTypes,

  pub grant_type: GrantTypes,

  pub user_ids: Vec<Uuid>,
}

impl Mutation {
  pub async fn update_resource_policy_impl(
    ctx: &GQLContext,
    updated_policy: UpdatePolicy,
  ) -> FieldResult<Policy> {
    update_policy(&ctx.pool, &ctx.auth0_user_id, updated_policy.into())
      .await
      .map(|db_policy| {
        db_policy
          .try_into()
          .expect("Unable to convert DBPolicy into Policy") // TODO: get rid of this
      })
      .map_err(FieldError::from)
  }
}
