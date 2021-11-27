use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::{
  graphql::schema::{
    policy::{GrantTypes, NewPolicy, PermissionTypes, PolicyTypes},
    project::NewProject,
  },
  services::db::{policy_service::create_policy, user_service::get_user_by_auth0_id},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DBProject {
  pub id: Uuid,

  pub name: String,

  pub user_ids: Vec<Uuid>,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

pub struct DBNewProject {
  pub name: String,
}

impl From<NewProject> for DBNewProject {
  fn from(new_project: NewProject) -> Self {
    DBNewProject {
      name: new_project.name,
    }
  }
}

pub async fn get_project(pool: impl PgExecutor<'_>, project_id: Uuid) -> Result<DBProject> {
  sqlx::query_as!(
    DBProject,
    r#"
    with
      _users_ids as (select user_id from user_access where object_type = 'Project' and object_id = $1)
    select
      id,
      name,
      array(select * from _users_ids) as "user_ids!",
      created_at,
      created_by,
      updated_at,
      updated_by
    from projects where id = $1;
    "#,
    project_id
  )
  .fetch_one(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_projects(
  pool: impl PgExecutor<'_>,
  project_ids: &[Uuid],
) -> Result<Vec<DBProject>> {
  sqlx::query_as!(
    DBProject,
    r#"
    with
      _users_ids as (select * from user_access where object_type = 'Project' and object_id = any($1))
    select
      projects.id as "id!",
      projects.name as "name!",
      array(select user_id from _users_ids where object_id = projects.id) as "user_ids!",
      projects.created_at as "created_at!",
      projects.created_by as "created_by!",
      projects.updated_at as "updated_at!",
      projects.updated_by as "updated_by!"
    from projects, users where projects.id = any($1);
    "#,
    project_ids
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn get_auth0_user_projects(
  pool: impl PgExecutor<'_>,
  auth0_id: &str,
) -> Result<Vec<DBProject>> {
  sqlx::query_as!(
    DBProject,
    //   r#"
    //   with
    //   _user as (select (id) from users where auth0id = $1),
    //   _user_project as (select (project_id) from user_project where user_id = (select id from _user))
    // select
    //   id as "id!",
    //   name as "name!",
    //   created_at as "created_at!",
    //   created_by as "created_by!",
    //   updated_at as "updated_at!",
    //   updated_by as "updated_by!"
    // from projects where id = any(select project_id from _user_project);
    //   "#,
    r#"
    with 
    _user as (select (id) from users where auth0id = $1),
    _user_project as 
      (select resource_id as project_id
       from policies
       where policy_type = 'Project' and user_ids @> array(select id from _user))
    select
    id as "id!",
    name as "name!",
    array(select user_id from user_access where object_id = projects.id) as "user_ids!",
    created_at as "created_at!",
    created_by as "created_by!",
    updated_at as "updated_at!",
    updated_by as "updated_by!"
  from projects where id = any(select project_id from _user_project);
    "#,
    auth0_id
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

pub async fn add_user_to_project(
  pool: impl PgExecutor<'_>,
  auth0_id: &str,
  user_id: Uuid,
  project_id: Uuid,
) -> Result<i32> {
  sqlx::query!(
    //   r#"
    // with _user as (select * from users where auth0id = $1)
    // insert into user_project (user_id, project_id, created_by, updated_by)
    // values ($2, $3, (select id from _user), (select id from _user))
    // "#,
    r#"
    with _user as (select * from users where auth0id = $1)
    insert into user_access (user_id, object_type, object_id, created_by, updated_by)
    values ($2, 'Project', $3, (select id from _user), (select id from _user))
  "#,
    auth0_id,
    user_id,
    project_id
  )
  .execute(pool)
  .await
  .map(|qr| qr.rows_affected() as i32)
  .map_err(anyhow::Error::from)
}

pub async fn create_project(
  pool: PgPool,
  auth0_id: &str,
  new_project: DBNewProject,
) -> Result<DBProject> {
  let mut tx = pool.begin().await?;

  let user = get_user_by_auth0_id(&mut tx, auth0_id).await?;

  let project = sqlx::query_as!(
    DBProject,
    r#"
    with 
      _user as (select * from users where auth0id = $1)
    insert into projects (name, created_by, updated_by) values ($2, (select id from _user), (select id from _user))
    returning
      id as "id!",
      name as "name!",
      array(select user_id from user_access where object_id = projects.id) as "user_ids!",
      created_at as "created_at!",
      created_by as "created_by!",
      updated_at as "updated_at!",
      updated_by as "updated_by!"
    ;
    "#,
    auth0_id,
    new_project.name,
  )
  .fetch_one(&mut tx)
  .await
  .map_err(anyhow::Error::from)?;

  // Dashboard Permissions
  let new_project_policy = NewPolicy {
    resource_id: project.id,
    policy_type: PolicyTypes::ProjectPolicy,
    permission_type: PermissionTypes::DashboardPermission,
    grant_type: GrantTypes::All,
    user_ids: vec![user.id],
  };
  create_policy(&mut tx, auth0_id, new_project_policy.into()).await?;

  // User Permissions
  let new_project_policy = NewPolicy {
    resource_id: project.id,
    policy_type: PolicyTypes::ProjectPolicy,
    permission_type: PermissionTypes::UserPermission,
    grant_type: GrantTypes::All,
    user_ids: vec![user.id],
  };
  create_policy(&mut tx, auth0_id, new_project_policy.into()).await?;

  add_user_to_project(&mut tx, auth0_id, user.id, project.id).await?;

  tx.commit().await?;

  Ok(project)
}

// pub async fn share_project(
//   pool: PgPool,
//   auth0_id: &str,
//   project_id: Uuid,
//   user_ids: Vec<Uuid>,
// ) -> Result<i32> {
//   let mut tx = pool.begin().await?;
//   let mut res = 0;

//   // TODO: Can't run async closure with &mut
//   // For now, adding a user to a project directly adds the user to all dashboards as well
//   let dashboards = get_project_dashboards(&mut tx, &[project_id]).await?;
//   let dashboard_ids = dashboards
//     .into_iter()
//     .map(|db_dashboard| db_dashboard.id)
//     .collect::<Vec<Uuid>>();
//   for user_id in user_ids {
//     res += add_user_to_project(&mut tx, auth0_id, user_id, project_id).await?;
//     res += add_user_to_dashboards(&mut tx, auth0_id, user_id, &dashboard_ids).await?;
//   }

//   tx.commit().await?;

//   Ok(res)
// }
