use actix_web::{ web, HttpResponse, dev, Error };

async fn get_health() -> Result<HttpResponse, Error> {
  Ok(HttpResponse::Ok().body(String::from("Hello from the other side!")))
}

pub fn get_utility_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/utility")
    .route("/health", web::get().to(get_health))
}