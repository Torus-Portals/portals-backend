use diesel::prelude::*;
use actix_web::{ web, HttpResponse, dev, Error };
use futures::future::{ Future, ok as fut_ok };

use crate::models::user::{ User, NewUser };
use crate::db::Pool;

use crate::schema;

pub fn get_user() -> impl Future<Item = HttpResponse, Error = Error> {
  fut_ok(HttpResponse::Ok().body("This is the get_user route!!!!"))
}

pub fn create_user(
  new_user: NewUser,
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  web::block(move || -> Result<User, diesel::result::Error> {
    let conn: &PgConnection = &pool.get().unwrap();

    diesel::insert_into(schema::users::table)
      .values(new_user)
      .get_result::<User>(conn)
  })
  .then(|res| match res {
    Ok(user) => fut_ok(HttpResponse::Ok().json(user)),
    Err(_) => fut_ok(HttpResponse::InternalServerError().into())
  })
}

pub fn get_user_resource() -> impl dev::HttpServiceFactory + 'static {
  web::resource("/user")
    .route(web::get().to_async(get_user))
    .route(web::post().to_async(create_user))
}