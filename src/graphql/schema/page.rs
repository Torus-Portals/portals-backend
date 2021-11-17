use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use uuid::Uuid;

use super::Mutation;
use super::Query;

use crate::graphql::context::GQLContext;
use crate::services::db::page_service::{
  create_page, get_dashboard_pages, get_page, update_page, delete_page, DBPage,
};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
  pub id: Uuid,

  pub name: String,

  pub project_id: Uuid,

  pub dashboard_id: Uuid,

  pub grid: Grid,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl From<DBPage> for Page {
  fn from(page: DBPage) -> Self {
    let grid: Grid =
      serde_json::from_value(page.grid).expect("Unable to convert Page grid from serde value");

    Page {
      id: page.id,
      name: page.name,
      grid,
      project_id: page.project_id,
      dashboard_id: page.dashboard_id,
      created_at: page.created_at,
      created_by: page.created_by,
      updated_at: page.updated_at,
      updated_by: page.updated_by,
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewPage {
  pub name: String,

  pub project_id: Uuid,

  pub dashboard_id: Uuid,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct UpdatePage {
  pub id: Uuid,

  pub name: Option<String>,

  pub project_id: Option<Uuid>,

  pub dashboard_id: Option<Uuid>,

  pub grid: Option<GridInput>,
}

#[derive(GraphQLObject, Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
  pub id: Uuid,

  pub rows: Vec<GridRow>,
}

impl Grid {
  pub fn new() -> Self {
    Grid {
      id: Uuid::new_v4(),
      rows: vec![],
    }
  }
}

#[derive(GraphQLObject, Debug, Clone, Serialize, Deserialize)]
pub struct GridRow {
  pub id: Uuid,

  pub height: i32,

  pub blocks: Vec<GridBlock>,
}

#[derive(GraphQLObject, Debug, Clone, Serialize, Deserialize)]
pub struct GridBlock {
  pub id: Uuid,

  block_id: Option<Uuid>,

  is_empty: bool,

  start: f64,

  end: f64,
}

#[derive(GraphQLInputObject, Debug, Clone, Serialize, Deserialize)]
pub struct GridInput {
  pub id: Uuid,

  pub rows: Vec<GridRowInput>,
}

#[derive(GraphQLInputObject, Debug, Clone, Serialize, Deserialize)]
pub struct GridRowInput {
  pub id: Uuid,

  pub height: i32,

  pub blocks: Vec<GridBlockInput>,
}

#[derive(GraphQLInputObject, Debug, Clone, Serialize, Deserialize)]
pub struct GridBlockInput {
  pub id: Uuid,

  pub block_id: Option<Uuid>,

  pub is_empty: bool,

  pub start: f64,

  pub end: f64,
}

impl Query {
  pub async fn page_impl(ctx: &GQLContext, page_id: Uuid) -> FieldResult<Page> {
    get_page(&ctx.pool, page_id)
      .await
      .map(|db_page| db_page.into())
      .map_err(FieldError::from)
  }

  pub async fn pages_impl(ctx: &GQLContext, dashboard_id: Uuid) -> FieldResult<Vec<Page>> {
    get_dashboard_pages(&ctx.pool, dashboard_id)
      .await
      .map(|db_pages| {
        db_pages
          .into_iter()
          .map(|p| p.into())
          .collect()
      })
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_page_impl(ctx: &GQLContext, new_page: NewPage) -> FieldResult<Page> {
    let local_pool = ctx.pool.clone();

    create_page(local_pool, &ctx.auth0_user_id, new_page.into())
      .await
      .map(|db_page| db_page.into())
      .map_err(FieldError::from)
  }

  pub async fn update_page_impl(ctx: &GQLContext, updated_page: UpdatePage) -> FieldResult<Page> {
    update_page(&ctx.pool, &ctx.auth0_user_id, updated_page.into())
      .await
      .map(|db_page| db_page.into())
      .map_err(FieldError::from)
  }

  pub async fn delete_page_impl(ctx: &GQLContext, page_id: Uuid) -> FieldResult<DateTime<Utc>> {
    delete_page(&ctx.pool, &ctx.auth0_user_id, page_id)
      .await
      .map_err(FieldError::from)
  }
}
