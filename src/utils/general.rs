// use actix_web::{ web, HttpResponse, Error };
// use serde::Serialize;

// pub async fn query_to_response<F, I, E>(func: F) -> Result<HttpResponse, Error> 
//   where F: FnOnce() -> Result<I, E> + Send + 'static,
//         I: Send + Serialize + 'static,
//         E: Send + std::fmt::Debug + 'static {
//   Ok(web::block(func)
//   .await
//   .map(|item| HttpResponse::Ok().json(item))
//   // TODO: Add much better handling of errors, give something useful to the clients.
//   .map_err(|_| HttpResponse::InternalServerError())?)
// }