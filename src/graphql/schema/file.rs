use chrono::{DateTime, Utc};
use juniper::{
  FieldError, FieldResult, GraphQLInputObject, GraphQLObject};
use uuid::Uuid;

use super::s3::{S3GetParams, S3PutParams, S3RequestData};
use super::Mutation;
use super::Query;
use crate::graphql::context::GQLContext;
use crate::services::db::file_service::{create_file, get_file, get_files, DBFile};

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
#[serde(rename_all = "camelCase")]
pub struct UploadFile {
  pub id: Uuid,

  pub key: Uuid,

  pub url: String,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub struct DownloadFile {
  pub url: String,
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
#[serde(rename_all = "camelCase")]
pub struct File {
  pub id: Uuid,

  pub project_id: Uuid,

  pub name: String,

  pub key: Uuid,

  pub extension: String,

  pub version: i32,

  pub size: i32,

  pub created_at: DateTime<Utc>,

  pub created_by: Uuid,

  pub updated_at: DateTime<Utc>,

  pub updated_by: Uuid,
}

impl From<DBFile> for File {
  fn from(db_file: DBFile) -> Self {
    Self {
      id: db_file.id,
      project_id: db_file.project_id,
      name: db_file.name,
      key: db_file.key,
      extension: db_file.extension,
      version: db_file.version,
      size: db_file.size,
      created_at: db_file.created_at,
      created_by: db_file.created_by,
      updated_at: db_file.updated_at,
      updated_by: db_file.updated_by,
    }
  }
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct NewFile {
  pub id: Uuid, // Unlike other objects, the id for files is generated client-side.

  pub project_id: Uuid,

  pub name: String,

  pub key: Uuid,

  pub extension: String,

  pub size: i32,
}

#[derive(GraphQLInputObject, Debug, Serialize, Deserialize)]
pub struct UploadFileParams {
  pub id: Uuid,

  pub project_id: Uuid,

  pub key: Uuid,
}

impl Query {
  pub async fn upload_file_impl(
    ctx: &GQLContext,
    params: UploadFileParams,
  ) -> FieldResult<UploadFile> {
    let s3 = ctx.s3.lock().await;
    let key = params.key.clone();
    let id = params.id.clone();

    // project/key/id
    let bucket = format!(
      "portals-dev-files-1/{}/{}/{}",
      params
        .project_id
        .to_string(),
      key.to_string(),
      id.to_string()
    );
    let url = s3
      .s3_presigned_url(S3RequestData::PutObject(S3PutParams {
        bucket,
        key: id.to_string(),
      }))
      .await?;

    Ok(UploadFile { id, key, url })
  }

  pub async fn download_file_impl(ctx: &GQLContext, file_id: Uuid) -> FieldResult<DownloadFile> {
    let s3 = ctx.s3.lock().await;

    let file = get_file(&ctx.pool, file_id).await?;

    let bucket = format!(
      "portals-dev-files-1/{}/{}/{}",
      file
        .project_id
        .to_string(),
      file.key.to_string(),
      file.id.to_string()
    );

    Ok(DownloadFile {
      url: s3
        .s3_presigned_url(S3RequestData::GetObject(S3GetParams {
          bucket,
          key: file.id.to_string(),
          name: file.name.clone(),
        }))
        .await?,
    })
  }

  pub async fn files_impl(ctx: &GQLContext, files: Vec<Uuid>) -> FieldResult<Vec<File>> {
    get_files(&ctx.pool, files)
      .await
      .map(|files| {
        files
          .into_iter()
          .map(|file| file.into())
          .collect()
      })
      .map_err(FieldError::from)
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
