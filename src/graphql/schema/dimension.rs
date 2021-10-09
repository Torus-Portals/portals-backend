use crate::services::db::dimension_service::{
  create_dimension, create_dimensions, get_dimension, get_dimensions, DBNewDimension,
};
use crate::{graphql::context::GQLContext, services::db::dimension_service::DBDimension};
use chrono::{DateTime, Utc};
use juniper::{
  FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, GraphQLUnion,
};
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::Mutation;
use super::Query;

use super::dimensions::google_sheets_column_dimension::GoogleSheetsColumnDimension;
use super::dimensions::google_sheets_row_dimension::GoogleSheetsRowDimension;
use super::dimensions::{
  basic_table_column_dimension::BasicTableColumnDimension,
  basic_table_row_dimension::BasicTableRowDimension, empty_dimension::EmptyDimension,
  owner_text_dimension::OwnerTextDimension, portal_member_dimension::PortalMemberDimension,
};

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum DimensionTypes {
  #[strum(serialize = "PortalMember")]
  #[graphql(name = "PortalMember")]
  PortalMember,

  #[strum(serialize = "BasicTableRow")]
  #[graphql(name = "BasicTableRow")]
  BasicTableRow,

  #[strum(serialize = "BasicTableColumn")]
  #[graphql(name = "BasicTableColumn")]
  BasicTableColumn,

  #[strum(serialize = "OwnerText")]
  #[graphql(name = "OwnerText")]
  OwnerText,

  #[strum(serialize = "GoogleSheetsRow")]
  #[graphql(name = "GoogleSheetsRow")]
  GoogleSheetsRow,

  #[strum(serialize = "GoogleSheetsColumn")]
  #[graphql(name = "GoogleSheetsColumn")]
  GoogleSheetsColumn,

  #[strum(serialize = "Empty")]
  #[graphql(name = "Empty")]
  Empty,
}

#[derive(Debug, Clone, GraphQLUnion, Serialize, Deserialize)]
pub enum GQLDimensions {
  BasicTableRow(BasicTableRowDimension),
  BasicTableColumn(BasicTableColumnDimension),
  PortalMember(PortalMemberDimension),
  OwnerText(OwnerTextDimension),
  GoogleSheetsRow(GoogleSheetsRowDimension),
  GoogleSheetsColumn(GoogleSheetsColumnDimension),
  Empty(EmptyDimension),
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct Dimension {
  pub id: Uuid,

  pub portal_id: Uuid,

  pub name: String,

  pub dimension_type: DimensionTypes,

  pub dimension_data: GQLDimensions,

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

    let dimension_data = match dim_type_str {
      "PortalMember" => {
        let d: PortalMemberDimension = serde_json::from_value(db_dimension.dimension_data)
          .expect("Can't deserialize PortalMemberDimension");
        GQLDimensions::PortalMember(d)
      }
      "BasicTableRow" => {
        let d: BasicTableRowDimension = serde_json::from_value(db_dimension.dimension_data)
          .expect("Can't deserialize BasicTableRowDimension");
        GQLDimensions::BasicTableRow(d)
      }
      "BasicTableColumn" => {
        let d: BasicTableColumnDimension = serde_json::from_value(db_dimension.dimension_data)
          .expect("Can't deserialize BasicTableColumnDimension");
        GQLDimensions::BasicTableColumn(d)
      }
      "OwnerText" => {
        let d: OwnerTextDimension = serde_json::from_value(db_dimension.dimension_data)
          .expect("Can't deserialize OwnerTextDimension");
        GQLDimensions::OwnerText(d)
      }
      "GoogleSheetsRow" => {
        let d: GoogleSheetsRowDimension = serde_json::from_value(db_dimension.dimension_data)
          .expect("Can't deserialize GoogleSheetsRowDimension");
        GQLDimensions::GoogleSheetsRow(d)
      }
      "GoogleSheetsColumn" => {
        let d: GoogleSheetsColumnDimension = serde_json::from_value(db_dimension.dimension_data)
          .expect("Can't deserialize GoogleSheetsColumnDimension");
        GQLDimensions::GoogleSheetsColumn(d)
      }
      "Empty" => {
        let d: EmptyDimension = serde_json::from_value(db_dimension.dimension_data)
          .expect("Can't deserialize EmptyDimension");
        GQLDimensions::Empty(d)
      }
      &_ => GQLDimensions::Empty(EmptyDimension { empty: true }),
    };

    Dimension {
      id: db_dimension.id,
      portal_id: db_dimension.portal_id,
      name: db_dimension.name,
      dimension_type,
      dimension_data,
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

  #[graphql(
    description = "For now dimension_data needs to be stringified, but it is type checked when parsed!"
  )]
  pub dimension_data: String,
}

impl Query {
  pub async fn dimension_impl(ctx: &GQLContext, dimension_id: Uuid) -> FieldResult<Dimension> {
    get_dimension(&ctx.pool, dimension_id)
      .await
      .map(|db_dimension| db_dimension.into())
      .map_err(FieldError::from)
  }

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
  pub async fn create_dimension_impl(
    ctx: &GQLContext,
    new_dimension: NewDimension,
  ) -> FieldResult<Dimension> {
    create_dimension(&ctx.pool, &ctx.auth0_user_id, new_dimension.into())
      .await
      .map(|db_dim| db_dim.into())
      .map_err(FieldError::from)
  }

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
