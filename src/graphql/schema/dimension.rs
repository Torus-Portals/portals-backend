use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLEnum, GraphQLObject, GraphQLUnion};
use std::str::FromStr;
use strum_macros::EnumString;
use crate::{graphql::context::GQLContext, services::db::dimension_service::DBDimension};
use uuid::Uuid;

use super::Query;


// #[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
// pub enum GQLDimensions {
//   BasicTableRow
//   BasicTableColumn
//   Empty
// }

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString)]
pub enum DimensionTypes {
  BasicTableRow,
  BasicTableColumn,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct Dimension {
  pub id: Uuid,

  #[serde(rename = "portalId")]
  pub portal_id: Uuid,

  pub name: String,

  #[serde(rename = "dimensionType")]
  pub dimension_type: DimensionTypes,

  // pub meta: serde_json::Value,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

impl From<DBDimension> for Dimension {
  fn from(db_dimension: DBDimension) -> Self {
    let dimension_type = DimensionTypes::from_str(db_dimension.dimension_type.as_str())
      .expect("Unable to convert dimension_type to enum variat");

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

impl Query {
  pub async fn dimensions_impl(ctx: &GQLContext, portal_id: Uuid) -> FieldResult<Vec<Dimension>> {
    ctx
    .db
    .get_dimensions(portal_id)
    .await
    .map(|dims| dims.into_iter().map(|d| d.into()).collect())
    .map_err(FieldError::from)
  }
}