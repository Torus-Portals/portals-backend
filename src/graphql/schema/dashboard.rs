use chrono::{DateTime, Utc};
use juniper::{FieldResult, FieldError, GraphQLObject};
use uuid::Uuid;

use super::Query;

use crate::graphql::context::GQLContext;
use crate::services::db::dashboard_service::{DBDashboard, get_project_dashboards};

#[derive(GraphQLObject, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
  pub id: Uuid,

  pub name: String,

  pub project_id: Uuid,

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
      project_id: dashboard.project_id,
      created_at: dashboard.created_at,
      created_by: dashboard.created_by,
      updated_at: dashboard.updated_at,
      updated_by: dashboard.updated_by,
    }
  }
}

impl Query {
  pub async fn dashboards_impl(ctx: &GQLContext, project_id: Uuid) -> FieldResult<Vec<Dashboard>> {
    get_project_dashboards(&ctx.pool, project_id)
    .await
    .map(|db_dashboards| {
      db_dashboards.into_iter().map(|dbd| dbd.into()).collect()
    })
    .map_err(FieldError::from)
  }
}