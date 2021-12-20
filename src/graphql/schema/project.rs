use chrono::{DateTime, Utc};
use juniper::{graphql_object, FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use uuid::Uuid;

use super::Mutation;
use super::Query;
use super::user::UserMetaInput;

use crate::graphql::context::GQLContext;

use crate::graphql::schema::dashboard::Dashboard;
use crate::graphql::schema::user::NewUser;
use crate::graphql::schema::user::User;
use crate::services::db::project_service::add_user_to_project;
use crate::services::db::project_service::share_project;
// use crate::services::db::project_service::share_project;
use crate::services::db::project_service::{
  create_project, get_auth0_user_projects, get_project, DBNewProject, DBProject,
};
use crate::services::db::user_service::{
  create_user_with_new_org, get_user_by_email, user_exists_by_email,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
  pub id: Uuid,

  pub name: String,

  pub user_ids: Vec<Uuid>,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

#[graphql_object(Context = GQLContext)]
impl Project {
  fn id(&self) -> Uuid {
    self.id
  }

  fn name(&self) -> &str {
    &self.name
  }

  fn user_ids(&self) -> &Vec<Uuid> {
    &self.user_ids
  }

  pub async fn dashboards(&self, context: &GQLContext) -> Vec<Dashboard> {
    let dashboard_map = context
      .dashboard_loader
      .load_many(vec![self.id.clone()])
      .await;

    let dashboards = match dashboard_map.get(&self.id) {
      Some(dashboards) => dashboards.clone(),
      None => vec![],
    };

    dashboards
  }

  pub async fn users(&self, context: &GQLContext) -> Vec<User> {
    let user_map = context
      .user_loader
      .load_many(vec![self.id.clone()])
      .await;

    let users = match user_map.get(&self.id) {
      Some(users) => users.clone(),
      None => vec![],
    };

    users
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

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub struct ProjectParts {
  pub projects: Vec<Project>,

  pub dashboards: Vec<Dashboard>,

  pub users: Vec<User>,
}

impl From<DBProject> for Project {
  fn from(project: DBProject) -> Self {
    Project {
      id: project.id,
      name: project.name,
      user_ids: project.user_ids,
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

  pub async fn share_project_impl(
    ctx: &GQLContext,
    project_id: Uuid,
    user_ids: Vec<Uuid>,
  ) -> FieldResult<i32> {
    let local_pool = ctx.pool.clone();
    
    share_project(local_pool, &ctx.auth0_user_id, project_id, user_ids)
      .await
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

  pub async fn add_user_to_project_impl(
    ctx: &GQLContext,
    user_email: String,
    project_id: Uuid,
  ) -> FieldResult<Project> {
    let exists = user_exists_by_email(&ctx.pool, &user_email).await?;

    let user = match exists {
      true => get_user_by_email(&ctx.pool, &user_email).await?,
      false => {
        let new_user = NewUser {
          name: String::new(),
          nickname: String::new(),
          email: user_email.to_owned(),
          meta: UserMetaInput::default(),
          org_ids: None,
          role_ids: None,
        };

        let user_and_org = create_user_with_new_org(
          ctx.pool.clone(),
          &ctx.auth0_user_id,
          new_user.into_db_new_user(String::new())?,
        )
        .await?;

        user_and_org.0
      }
    };

    let users_added_count =
      add_user_to_project(&ctx.pool, &ctx.auth0_user_id, user.id, project_id).await?;

    if users_added_count == 0 {
      return Err(FieldError::from("User not added to project"));
    }

    let project: Project = get_project(&ctx.pool, project_id)
    .await?
    .into();

    Ok(project)
  }
}