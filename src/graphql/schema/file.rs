use chrono::{DateTime, Utc};
use juniper::{
  FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, GraphQLUnion,
};
use serde_json;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::Mutation;
use super::Query;
use super::s3::{S3RequestData, S3PutParams};
use crate::graphql::context::GQLContext;
use crate::services::db::file_service::{create_file, DBFile};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
#[serde(rename_all = "camelCase")]
pub struct UploadFile {
  pub key: String,

  pub url: String,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
#[serde(rename_all = "camelCase")]
pub struct File {
  pub id: Uuid,

  pub name: String,

  pub key: String,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl From<DBFile> for File {
  fn from(db_file: DBFile) -> Self {
    Self {
      id: db_file.id,
      name: db_file.name,
      key: db_file.key,
      created_at: db_file.created_at,
      created_by: db_file.created_by,
      updated_at: db_file.updated_at,
      updated_by: db_file.updated_by,
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewFile {
  pub name: String,

  pub key: String,
}

impl Query {
  pub async fn upload_file_impl(ctx: &GQLContext) -> FieldResult<UploadFile> {
    let s3 = ctx.s3.lock().await;
    let key = Uuid::new_v4().to_string();
    let bucket = "dev-file-upload-test-1".to_string(); // TODO: get from config.
    let url = s3.s3_presigned_url(S3RequestData::PutObject(S3PutParams { bucket, key: key.clone() }))
    .await?;

    Ok(UploadFile {
      key,
      url,
    })
  }

  // pub async fn file_impl(ctx: &GQLContext, file_id: Uuid) -> FieldResult<File> {
  //   get_file(&ctx.pool, file_id)
  //     .await
  //     .map(|db_file| db_file.into())
  //     .map_err(FieldError::from)
  // }
}

impl Mutation {
  pub async fn create_file_impl(ctx: &GQLContext, new_file: NewFile) -> FieldResult<File> {
    create_file(&ctx.pool, &ctx.auth0_user_id, new_file.into())
      .await
      .map(|db_file| db_file.into())
      .map_err(FieldError::from)
  }
}
