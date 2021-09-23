use crate::services::db::dimension_service::{create_dimensions, get_dimensions, DBNewDimension};
use crate::{graphql::context::GQLContext, services::db::dimension_service::DBDimension};
use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject};
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::Mutation;
use super::Query;

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum DimensionTypes {
  #[strum(serialize = "BasicTable-row")]
  BasicTableRow,
  #[strum(serialize = "BasicTable-column")]
  BasicTableColumn,
  #[strum(serialize = "OwnerText")]
  OwnerText,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct Dimension {
  pub id: Uuid,

  pub portal_id: Uuid,

  pub name: String,

  pub dimension_type: DimensionTypes,

  // pub meta: serde_json::Value,
  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl From<DBDimension> for Dimension {
  fn from(db_dimension: DBDimension) -> Self {
    let dim_type_str = db_dimension
      .dimension_type
      .as_str();

    let dimension_type = DimensionTypes::from_str(dim_type_str)
      .expect("Unable to convert dimension_type to enum variant");

    Dimension {
      id: db_dimension.id,
      portal_id: db_dimension.portal_id,
      name: db_dimension.name,
      dimension_type,
      created_at: db_dimension.created_at,
      created_by: db_dimension.created_by,
      updated_at: db_dimension.updated_at,
      updated_by: db_dimension.updated_by,
    }
  }
}

#[derive(Debug, GraphQLInputObject, Serialize, Deserialize)]
pub struct NewDimension {
  pub portal_id: Uuid,

  pub name: String,

  pub dimension_type: DimensionTypes,
}

impl Query {
  pub async fn dimensions_impl(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Vec<Dimension>> {
    get_dimensions(&ctx.pool, portal_id)
      .await
      .map(|dimensions| {
        dimensions
          .into_iter()
          .map(|d| d.into())
          .collect()
      })
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_dimensions_impl(
    ctx: &GQLContext,
    dimensions: Vec<NewDimension>,
  ) -> FieldResult<Vec<Dimension>> {
    let db_dims = dimensions
      .into_iter()
      .map(|db_dim| db_dim.into())
      .collect::<Vec<DBNewDimension>>();

    create_dimensions(&ctx.pool, &ctx.auth0_user_id, db_dims)
      .await
      .map(|db_dimensions| {
        db_dimensions
          .into_iter()
          .map(|d| d.into())
          .collect()
      })
      .map_err(FieldError::from)
  }
}
