use chrono::{DateTime, Utc};
use juniper::{graphql_object, FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use uuid::Uuid;

use super::Query;

use crate::graphql::context::GQLContext;

use crate::services::db::project_service::{get_project, get_auth0_user_projects, DBProject};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
  pub id: Uuid,

  pub name: String,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl From<DBProject> for Project {
  fn from(project: DBProject) -> Self {
    Project {
      id: project.id,
      name: project.name,
      created_at: project.created_at,
      created_by: project.created_by,
      updated_at: project.updated_at,
      updated_by: project.updated_by,
    }
  }
}

impl Query {
  pub async fn project_impl(ctx: &GQLContext, project_id: Uuid) -> FieldResult<Project> {
    get_project(&ctx.pool, project_id)
      .await
      .map(|db_project| db_project.into())
      .map_err(FieldError::from)
  }

  pub async fn projects_impl(ctx: &GQLContext) -> FieldResult<Vec<Project>> {
    get_auth0_user_projects(&ctx.pool, &ctx.auth0_user_id)
    .await
    .map(|db_projects| {
      db_projects.into_iter().map(|p| p.into())
      .collect()
    })
    .map_err(FieldError::from)
  }
}
