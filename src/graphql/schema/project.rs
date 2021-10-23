use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use uuid::Uuid;

use super::Mutation;
use super::Query;

use crate::graphql::context::GQLContext;

use crate::services::db::project_service::{
  create_project, get_auth0_user_projects, get_project, DBNewProject, DBProject,
};

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

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewProject {
  pub name: String,
}

impl From<DBNewProject> for NewProject {
  fn from(db_new_project: DBNewProject) -> Self {
    NewProject {
      name: db_new_project.name,
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
        db_projects
          .into_iter()
          .map(|p| p.into())
          .collect()
      })
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_project_impl(
    ctx: &GQLContext,
    new_project: NewProject,
  ) -> FieldResult<Project> {
    let local_pool = ctx.pool.clone();

    create_project(local_pool, &ctx.auth0_user_id, new_project.into())
      .await
      .map(|db_project| db_project.into())
      .map_err(FieldError::from)
  }
}
