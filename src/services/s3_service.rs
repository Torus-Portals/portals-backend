use std::{sync::Arc, time::Duration};

use actix_web::{dev, web, Error, HttpResponse};
use futures::lock::Mutex;
use rusoto_core::{
  credential::{ProfileProvider, ProvideAwsCredentials},
  Region,
};
use rusoto_s3::{
  util::{PreSignedRequest, PreSignedRequestOption},
  CreateMultipartUploadRequest, GetObjectRequest, PutObjectRequest, S3Client, UploadPartRequest,
  S3,
};

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

#[derive(Clone, Debug)]
pub struct S3Service {}

impl S3Service {
  pub fn new() -> Self {
    S3Service {}
  }

  pub async fn s3_presigned_url(
    &self,
    request_data: S3RequestData,
  ) -> Result<String, anyhow::Error> {
    let region = Region::ApSoutheast1;
    let creds_profile = ProfileProvider::new().unwrap();
    let creds = creds_profile.credentials().await.unwrap();
    let presigned_req_option = PreSignedRequestOption {
      expires_in: Duration::new(3600, 0),
    };

    let url = match request_data {
      S3RequestData::GetObject(params) => {
        let req = GetObjectRequest {
          bucket: params.bucket,
          key: params.key,
          ..Default::default()
        };

        req.get_presigned_url(&region, &creds, &presigned_req_option)
      }
      S3RequestData::PutObject(params) => {
        let req = PutObjectRequest {
          bucket: params.bucket,
          key: params.key,
          ..Default::default()
        };

        req.get_presigned_url(&region, &creds, &presigned_req_option)
      }
      S3RequestData::UploadPart(params) => {
        // 1. Initialized multipart upload, retrieve upload_id
        let client = S3Client::new(Region::ApSoutheast1);
        let create_multipart_req = CreateMultipartUploadRequest {
          bucket: params.bucket.clone(),
          key: params.bucket.clone(),
          ..Default::default()
        };

        let create_multipart_res = client.create_multipart_upload(create_multipart_req).await?;
        dbg!(&create_multipart_res);
        let upload_id = create_multipart_res.upload_id.unwrap();
        println!("{}", upload_id);

        // 2. Upload parts
        let upload_part_urls: Vec<String> = (0..params.parts)
          .into_iter()
          .map(|part| UploadPartRequest {
            bucket: params.bucket.clone(),
            key: params.key.clone(),
            // body: Some(params.body.into()),
            part_number: part + 1,
            upload_id: upload_id.clone(),
            ..Default::default()
          })
          .map(|part_req| part_req.get_presigned_url(&region, &creds, &presigned_req_option))
          .collect();
        dbg!(&upload_part_urls);

        // TODO: 3. Complete multipart upload

        serde_json::to_string(&upload_part_urls)?
      }
    };

    Ok(url)
  }
}

pub async fn put_s3_presigned_url(
  s3: web::Data<Arc<Mutex<S3Service>>>,
  params: web::Json<S3PutParams>,
) -> Result<HttpResponse, Error> {
  let s3_service = s3.lock().await;
  let params = params.into_inner();
  let presigned_url = s3_service
    .s3_presigned_url(S3RequestData::PutObject(params))
    .await
    .ok()
    .unwrap();

  Ok(HttpResponse::Ok().body(presigned_url))
}

pub async fn upload_s3_presigned_url(
  s3: web::Data<Arc<Mutex<S3Service>>>,
  params: web::Json<S3UploadParams>,
) -> Result<HttpResponse, Error> {
  let s3_service = s3.lock().await;
  let params = params.into_inner();
  let presigned_url = s3_service
    .s3_presigned_url(S3RequestData::UploadPart(params))
    .await
    .ok()
    .unwrap();

  Ok(HttpResponse::Ok().body(presigned_url))
}

pub async fn get_s3_presigned_url(
  s3: web::Data<Arc<Mutex<S3Service>>>,
  params: web::Query<S3GetParams>,
) -> Result<HttpResponse, Error> {
  let s3_service = s3.lock().await;
  dbg!(&params);
  let params = params.into_inner();
  let presigned_url = s3_service
    .s3_presigned_url(S3RequestData::GetObject(params))
    .await
    .ok()
    .unwrap();

  Ok(HttpResponse::Ok().body(presigned_url))
}

pub fn get_s3_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/s3")
    .route("/upload", web::post().to(put_s3_presigned_url))
    .route("/get", web::get().to(get_s3_presigned_url))
    .route("/multipart", web::post().to(upload_s3_presigned_url))
}
