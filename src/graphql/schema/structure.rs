use chrono::{DateTime, NaiveDateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLEnum, GraphQLObject, GraphQLUnion};
use std::str::FromStr;
use strum_macros::EnumString;
use uuid::Uuid;

use crate::{graphql::context::GQLContext, services::db::structure_service::DBStructure};

use super::Query;

#[derive(Debug, Clone, GraphQLUnion, Serialize, Deserialize)]
pub enum GQLStructures {
  Grid(GridStructure),
  Empty(EmptyStructure),
}

#[derive(Debug, Serialize, Deserialize, Clone, GraphQLEnum, EnumString)]
pub enum StructureTypes {
  Grid,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize, Clone)]
pub struct Structure {
  pub id: Uuid,

  pub structure_type: StructureTypes,

  pub structure_data: GQLStructures,

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
      structure_data: GQLStructures::Empty(EmptyStructure {
        structure_type: String::from("nothing"),
      }),
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

    let structure_data = match db_structure
      .structure_type
      .as_str()
    {
      "Grid" => {
        let s = serde_json::from_value(db_structure.structure_data)
          .unwrap_or_else(|_| GridStructure { rows: vec![] });

        GQLStructures::Grid(s)
      }
      &_ => GQLStructures::Empty(EmptyStructure {
        structure_type: String::from("nothing"),
      }),
    };

    Structure {
      id: db_structure.id,
      structure_type,
      structure_data,
      created_at: db_structure.created_at,
      created_by: db_structure.created_by,
      updated_at: db_structure.updated_at,
      updated_by: db_structure.updated_by,
    }
  }
}

#[derive(GraphQLObject, Debug, Clone, Serialize, Deserialize)]
pub struct GridStructureRow {
  pub height: i32,

  pub blocks: Vec<Uuid>,

  pub widths: Vec<String>,
}

#[derive(GraphQLObject, Debug, Clone, Serialize, Deserialize)]
pub struct GridStructure {
  pub rows: Vec<GridStructureRow>,
}

#[derive(GraphQLObject, Debug, Clone, Serialize, Deserialize)]
pub struct EmptyStructure {
  structure_type: String,
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
