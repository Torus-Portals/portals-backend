use chrono::{DateTime, NaiveDateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLEnum, GraphQLObject};
use std::str::FromStr;
use strum_macros::EnumString;
use uuid::Uuid;

use crate::{graphql::context::GQLContext, services::db::structure_service::DBStructure};

use super::Query;

#[derive(Debug, Serialize, Deserialize, Clone, GraphQLEnum, EnumString)]
pub enum StructureTypes {
  Grid,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize, Clone)]
pub struct Structure {
  pub id: Uuid,

  pub structure_type: StructureTypes,

  #[serde(rename = "createdAt")]
  pub created_at: DateTime<Utc>,

  #[serde(rename = "createdBy")]
  pub created_by: Uuid,

  #[serde(rename = "updatedAt")]
  pub updated_at: DateTime<Utc>,

  #[serde(rename = "updatedBy")]
  pub updated_by: Uuid,
}

impl Default for Structure {
  fn default() -> Self {
    Structure {
      id: Uuid::new_v4(),
      structure_type: StructureTypes::Grid,
      created_at: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
      created_by: Uuid::new_v4(),
      updated_at: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
      updated_by: Uuid::new_v4(),
    }
  }
}

impl From<DBStructure> for Structure {
  fn from(db_structure: DBStructure) -> Self {
    let structure_type = StructureTypes::from_str(
      db_structure
        .structure_type
        .as_str(),
    )
    .expect("unable to convert structure_type string to enum variant");

    Structure {
      id: db_structure.id,
      structure_type,
      created_at: db_structure.created_at,
      created_by: db_structure.created_by,
      updated_at: db_structure.updated_at,
      updated_by: db_structure.updated_by,
    }
  }
}

impl Query {
  pub async fn structure_impl(ctx: &GQLContext, structure_id: Uuid) -> FieldResult<Structure> {
    ctx
      .db
      .get_structure(structure_id)
      .await
      .map(|db_structure| db_structure.into())
      .map_err(FieldError::from)
  }
}
