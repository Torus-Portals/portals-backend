use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use uuid::Uuid;

use super::Mutation;
use super::Query;

use crate::graphql::context::GQLContext;
use crate::services::db::dashboard_service::share_dashboard;
use crate::services::db::dashboard_service::{
  create_dashboard, get_dashboard, get_project_dashboards, update_dashboard, DBDashboard,
  DBNewDashboard,
  DBUpdateDashboard
};

#[derive(GraphQLObject, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
  pub id: Uuid,

  pub name: String,

  pub project_id: Uuid,

  // Storing page_ids on the dashboard to simplify updating the page tab indexes
  // such as if a page is deleted, or the order is changed.
  pub page_ids: Vec<Uuid>,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl From<DBDashboard> for Dashboard {
  fn from(dashboard: DBDashboard) -> Self {
    Dashboard {
      id: dashboard.id,
      name: dashboard.name,
      page_ids: dashboard.page_ids,
      project_id: dashboard.project_id,
      created_at: dashboard.created_at,
      created_by: dashboard.created_by,
      updated_at: dashboard.updated_at,
      updated_by: dashboard.updated_by,
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)] 
#[serde(rename_all = "camelCase")]
pub struct NewDashboard {
  pub name: String,

  pub project_id: Uuid,
}

impl From<DBNewDashboard> for NewDashboard {
  fn from(db_new_dashboard: DBNewDashboard) -> Self {
    NewDashboard {
      name: db_new_dashboard.name,
      project_id: db_new_dashboard.project_id,
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)] 
pub struct UpdateDashboard {
  pub id: Uuid,

  pub name: Option<String>,

  pub project_id: Option<Uuid>,

  pub page_ids: Option<Vec<Uuid>>,
}

impl From<DBUpdateDashboard> for UpdateDashboard {
  fn from(db_update_dashboard: DBUpdateDashboard) -> Self {
    UpdateDashboard {
      id: db_update_dashboard.id,
      name: db_update_dashboard.name,
      project_id: db_update_dashboard.project_id,
      page_ids: db_update_dashboard.page_ids,
    }
  }
}

impl Query {
  pub async fn dashboard_impl(ctx: &GQLContext, dashboard_id: Uuid) -> FieldResult<Dashboard> {
    get_dashboard(&ctx.pool, dashboard_id)
      .await
      .map(|db_d| db_d.into())
      .map_err(FieldError::from)
  }

  pub async fn dashboards_impl(ctx: &GQLContext, project_id: Uuid) -> FieldResult<Vec<Dashboard>> {
    get_project_dashboards(&ctx.pool, [project_id].as_ref())
      .await
      .map(|db_dashboards| {
        db_dashboards
          .into_iter()
          .map(|dbd| dbd.into())
          .collect()
      })
      .map_err(FieldError::from)
  }
  
  pub async fn share_dashboard_impl(ctx: &GQLContext, dashboard_id: Uuid, user_ids: Vec<Uuid>) -> FieldResult<i32> {
    let local_pool = ctx.pool.clone();

    share_dashboard(local_pool, &ctx.auth0_user_id, dashboard_id, user_ids)
      .await
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_dashboard_impl(
    ctx: &GQLContext,
    new_dashboard: NewDashboard,
  ) -> FieldResult<Dashboard> {
    let local_pool = ctx.pool.clone();

    create_dashboard(local_pool, &ctx.auth0_user_id, new_dashboard.into())
      .await
      .map(|db_d| db_d.into())
      .map_err(FieldError::from)
  }

  pub async fn update_dashboard_impl(
    ctx: &GQLContext,
    updated_dashboard: UpdateDashboard,
  ) -> FieldResult<Dashboard> {
    update_dashboard(&ctx.pool, &ctx.auth0_user_id, updated_dashboard.into())
      .await
      .map(|db_d| db_d.into())
      .map_err(FieldError::from)
  }
}
