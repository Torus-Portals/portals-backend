use std::{
  convert::{TryFrom, TryInto},
  str::FromStr,
};

use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject};
use strum_macros::{Display, EnumIter, EnumString};
use uuid::Uuid;

use crate::{graphql::context::GQLContext, services::db::policy_service::{DBPolicy, check_permission, resources_perms, update_policy, user_permissions, user_resource_perms}};

use super::{Mutation, Query};

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum PolicyTypes {
  #[strum(serialize = "Project")]
  #[graphql(name = "Project")]
  ProjectPolicy,

  #[strum(serialize = "Dashboard")]
  #[graphql(name = "Dashboard")]
  DashboardPolicy,

  #[strum(serialize = "Page")]
  #[graphql(name = "Page")]
  PagePolicy,

  #[strum(serialize = "Block")]
  #[graphql(name = "Block")]
  BlockPolicy,
}

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum PermissionTypes {
  #[strum(serialize = "Dashboard")]
  #[graphql(name = "Dashboard")]
  DashboardPermission,

  #[strum(serialize = "Page")]
  #[graphql(name = "Page")]
  PagePermission,

  #[strum(serialize = "Block")]
  #[graphql(name = "Block")]
  BlockPermission,

  #[strum(serialize = "User")]
  #[graphql(name = "User")]
  UserPermission,
}

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, EnumIter, Display)]
pub enum GrantTypes {
  #[strum(serialize = "All")]
  #[graphql(name = "All")]
  All,

  #[strum(serialize = "Create")]
  #[graphql(name = "Create")]
  Create,

  #[strum(serialize = "Read")]
  #[graphql(name = "Read")]
  Read,

  #[strum(serialize = "Update")]
  #[graphql(name = "Update")]
  Update,

  #[strum(serialize = "Delete")]
  #[graphql(name = "Delete")]
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
  pub resource_id: Uuid,

  pub policy_type: PolicyTypes,

  pub permission_type: PermissionTypes,

  pub grant_type: GrantTypes,

  pub user_ids: Vec<Uuid>,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct UserPermissionInput {
  pub resource_id: Uuid,

  pub user_id: Uuid,

  pub grant_type: GrantTypes,
}

impl Query {
  pub async fn get_user_permissions_impl(
    ctx: &GQLContext,
    user_id: Uuid,
  ) -> FieldResult<Vec<Policy>> {
    user_permissions(&ctx.pool, user_id)
      .await
      .and_then(|db_policies| {
        Ok(
          db_policies
            .into_iter()
            .map(|db_policy| {
              db_policy
                .try_into()
                .expect("Failed to convert DBPolicy into Policy")
            })
            .collect::<Vec<Policy>>(),
        )
      })
      .map_err(FieldError::from)
  }

  pub async fn get_user_resource_perms_impl(
    ctx: &GQLContext,
    user_id: Uuid,
    resource_id: Uuid,
  ) -> FieldResult<Vec<Policy>> {
    user_resource_perms(&ctx.pool, user_id, resource_id)
    .await
    .and_then(|db_policies| {
      Ok(
        db_policies
          .into_iter()
          .map(|db_policy| {
            db_policy
              .try_into()
              .expect("Failed to convert DBPolicy into Policy")
          })
          .collect::<Vec<Policy>>(),
      )
    })
    .map_err(FieldError::from)
  }  

  pub async fn check_user_permission_impl(
    ctx: &GQLContext,
    user_access: UserPermissionInput,
  ) -> FieldResult<bool> {
    check_permission(
      &ctx.pool,
      user_access.resource_id,
      user_access.user_id,
      user_access
        .grant_type
        .to_string(),
    )
    .await
    .map_err(FieldError::from)
  }

  pub async fn resources_perms_impl(
    ctx: &GQLContext,
    resource_ids: Vec<Uuid>,
  ) -> FieldResult<Vec<Policy>> {
    resources_perms(&ctx.pool, resource_ids)
    .await
    .and_then(|db_policies| {
      Ok(
        db_policies
          .into_iter()
          .map(|db_policy| {
            db_policy
              .try_into()
              .expect("Failed to convert DBPolicy into Policy")
          })
          .collect::<Vec<Policy>>(),
      )
    })
    .map_err(FieldError::from)
  }
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
