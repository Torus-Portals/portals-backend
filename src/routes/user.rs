use actix_web::{ web, get, Responder, HttpResponse, dev, Error };
use futures::future::{ Future, ok as fut_ok };

pub fn get_user() -> impl Future<Item = HttpResponse, Error = Error> {
  fut_ok(HttpResponse::Ok().body("This is the get_user route!!!!"))
}

pub fn get_user_resource() -> impl dev::HttpServiceFactory + 'static {
  web::resource("/user")
    .route(web::get().to_async(get_user))
}