// use actix_web::{ web, HttpResponse, dev, get };

// #[get("/health")]
// pub async fn get_health() -> Result<HttpResponse, actix_web::Error> {
//   Ok(HttpResponse::Ok().body(String::from("Hello from the other side!")))
// }

// // pub fn get_utility_routes() -> impl dev::HttpServiceFactory + 'static {
// //   web::scope("/utility")
// //     .service(get_health)
// // }