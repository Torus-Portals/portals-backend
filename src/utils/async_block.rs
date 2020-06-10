use actix_web::error::BlockingError;
use diesel::result::{ Error as DieselError };
use actix_web::{ web, HttpResponse, Error };
use serde::Serialize;

#[derive(Debug)]
pub enum BlockError {
  InternalServerError(String),
  DBQueryError(String),
  BadRequest(String),
}

impl From<DieselError> for BlockError {
  fn from(err: DieselError) -> Self {
    BlockError::DBQueryError("crap".to_string())
  }
}

#[derive(Debug)]
pub enum BlockResponse<T> {
  JSON(T),
  Empty,
}

pub type BlockResult<T> = Result<BlockResponse<T>, BlockError>;

pub async fn async_block<F, I>(func: F) -> Result<HttpResponse, Error> 
  where F: FnOnce() -> BlockResult<I> + Send + 'static,
        I: Send + Serialize + 'static {
  Ok(web::block(func)
  .await
  .map(|block_resp| {
    match block_resp {
      BlockResponse::JSON(json) => HttpResponse::Ok().json(json),
      BlockResponse::Empty => HttpResponse::Ok().finish(),
    }
  })
  .map_err(|err: BlockingError<BlockError> | -> HttpResponse {
    match err {
      BlockingError::Error(block_err) => {
        match block_err {
          BlockError::InternalServerError(_message) => {
            HttpResponse::InternalServerError().finish()
          },
          BlockError::DBQueryError(_message) => {
            HttpResponse::InternalServerError().finish()
          },
          BlockError::BadRequest(message) => {
            HttpResponse::BadRequest().body(message)
          }
        }
      },
      BlockingError::Canceled => {
        HttpResponse::InternalServerError().finish()
      }
    }
  })?)
}