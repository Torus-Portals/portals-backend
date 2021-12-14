use juniper::FieldResult;

use crate::graphql::context::GQLContext;
use crate::graphql::schema::Query;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum S3RequestData {
  GetObject(S3GetParams),
  PutObject(S3PutParams),
  UploadPart(S3UploadParams),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3GetParams {
  pub bucket: String,
  pub key: String,
  pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3PutParams {
  pub bucket: String,
  pub key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3UploadParams {
  pub bucket: String,
  pub key: String,
  pub parts: i64,
}

impl Query {
  pub async fn s3_upload_presigned_url_impl(
    ctx: &GQLContext,
    bucket: String,
    key: String,
  ) -> FieldResult<String> {
    let s3 = ctx.s3.lock().await;

    Ok(
      s3.s3_presigned_url(S3RequestData::PutObject(S3PutParams { bucket, key }))
        .await?,
    )
  }

  pub async fn s3_download_presigned_url_impl(
    ctx: &GQLContext,
    bucket: String,
    key: String,
  ) -> FieldResult<String> {
    let s3 = ctx.s3.lock().await;

    Ok(
      s3.s3_presigned_url(S3RequestData::GetObject(S3GetParams { bucket, key, name: "Don't use this".to_string() }))
        .await?,
    )
  }
}
