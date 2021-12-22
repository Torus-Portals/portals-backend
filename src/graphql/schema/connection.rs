use chrono::{DateTime, Utc};
use juniper::{FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject};
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use crate::graphql::context::GQLContext;
use crate::services::db::connection_service::{
  create_connection, get_connections, DBConnection};

use super::Mutation;
use super::Query;

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum ConnectionDestinationTypes {
  #[strum(serialize = "Block")]
  #[graphql(name = "Block")]
  Block,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct Connection {
  pub id: Uuid,

  pub name: String,

  pub block_id: Uuid,

  pub source_id: Option<Uuid>,

  pub sourcequery_id: Option<Uuid>,

  pub destination_id: Option<Uuid>,

  pub destination_type: Option<ConnectionDestinationTypes>,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl From<DBConnection> for Connection {
  fn from(db_connection: DBConnection) -> Self {
    let destination_type = db_connection
      .destination_type
      .map(|destination_type| {
        ConnectionDestinationTypes::from_str(&destination_type)
          .expect("Unable to convert destination_type string to enum variant")
      });

    Connection {
      id: db_connection.id,
      name: db_connection.name,
      block_id: db_connection.block_id,
      source_id: db_connection.source_id,
      sourcequery_id: db_connection.sourcequery_id,
      destination_id: db_connection.destination_id,
      destination_type,
      created_at: db_connection.created_at,
      created_by: db_connection.created_by,
      updated_at: db_connection.updated_at,
      updated_by: db_connection.updated_by,
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewConnection {
  pub name: String,

  pub block_id: Uuid,

  pub source_id: Option<Uuid>,

  pub sourcequery_id: Option<Uuid>,

  pub destination_id: Option<Uuid>,

  pub destination_type: Option<String>,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct UpdateConnection {
  pub id: Uuid,

  pub name: Option<String>,

  pub source_id: Option<Uuid>,

  pub sourcequery_id: Option<Uuid>,

  pub destination_id: Option<Uuid>,

  pub destination_type: Option<String>,
}

impl Query {
  pub async fn connections_impl(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Vec<Connection>> {
    get_connections(&ctx.pool, block_id)
      .await
      .map(|db_c| {
        db_c
          .into_iter()
          .map(|db_c| db_c.into())
          .collect()
      })
      .map_err(FieldError::from)
  }
}

impl Mutation {
  pub async fn create_connection_impl(
    ctx: &GQLContext,
    new_connection: NewConnection,
  ) -> FieldResult<Connection> {
    create_connection(&ctx.pool, &ctx.auth0_user_id, new_connection.into())
      .await
      .map(|db_c| db_c.into())
      .map_err(FieldError::from)
  }

  pub async fn update_connection_impl(
    _ctx: &GQLContext,
    _updated_connection: UpdateConnection,
  ) -> FieldResult<Connection> {
    unimplemented!()
  }
}
