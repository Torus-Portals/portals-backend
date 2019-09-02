use diesel::prelude::*;
use actix_web::{ web, HttpResponse, dev, Error };
use futures::future::{ Future, ok as fut_ok };

use crate::models::user::{ User, NewUser, UpdateUser, Auth0UserId };
use crate::db::Pool;

use crate::schema;

use crate::services::auth0_service::get_auth0_user;

#[derive(Deserialize)]
pub struct UserIdPath {
  user_id: i32
}

pub fn get_requesting_user(
  auth0_user_id: Auth0UserId,
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  let pool1 = pool.clone();
  // let pool2 = pool.clone();
  web::block(move || -> Result<User, diesel::result::Error> {
    use schema::users::dsl::*;

    let conn: &PgConnection = &pool1.get().unwrap();

    // Try to find user by auth0 id from jwt
    let res = users.filter(auth0id.eq(&auth0_user_id.id))
      .get_result::<User>(conn);

    match res {
        // If found, return
        Ok(user) => Ok(user),
        Err(err) => {
          // If user isn't found in db, see if user is in auth0
          match get_auth0_user(&auth0_user_id.id) {
            Ok(auth0_user) => {
              let new_user = NewUser {
                auth0id: auth0_user.user_id,
                name: auth0_user.name,
                nickname: auth0_user.nickname,
                email: auth0_user.email,
              };

              diesel::insert_into(users)
                .values(new_user)
                .get_result::<User>(conn)
            },
            Err(_) => {
              Err(err)
            }
          }
        }
      }
  })
  .then(|res| match res {
    Ok(user) => fut_ok(HttpResponse::Ok().json(user)),
    Err(_) => fut_ok(HttpResponse::InternalServerError().into())
  })
}

pub fn get_user_by_id(
  path: web::Path<UserIdPath>,
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  web::block(move || -> diesel::QueryResult<User> {
    let conn: &PgConnection = &pool.get().unwrap();

    schema::users::table
      .filter(schema::users::dsl::id.eq(path.user_id))
      .get_result::<User>(conn)
  })
  .then(|res| match res {
    Ok(user) => fut_ok(HttpResponse::Ok().json(user)),
    Err(_) => fut_ok(HttpResponse::InternalServerError().into())
  })
}

pub fn create_user(
  new_user: NewUser,
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  web::block(move || -> Result<User, diesel::result::Error> {
    use schema::users::dsl::*;

    let conn: &PgConnection = &pool.get().unwrap();

    diesel::insert_into(users)
      .values(new_user)
      .get_result::<User>(conn)
  })
  .then(|res| match res {
    Ok(user) => fut_ok(HttpResponse::Ok().json(user)),
    Err(_) => fut_ok(HttpResponse::InternalServerError().into())
  })
}

pub fn update_user(
  path: web::Path<UserIdPath>,
  updated_user: UpdateUser,
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  web::block(move || -> Result<User, diesel::result::Error> {
    use schema::users::dsl::*;

    let conn: &PgConnection = &pool.get().unwrap();

    diesel::update(users.find(path.user_id))
      .set(&updated_user)
      .get_result(conn)
  })
  .then(|res| match res {
    Ok(user) => fut_ok(HttpResponse::Ok().json(user)),
    Err(err) => {
      println!("{}", err);
      fut_ok(HttpResponse::InternalServerError().into())
    }
  })
}

pub fn get_user_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/users")
    .route("", web::post().to_async(create_user))
    .route("/user", web::get().to_async(get_requesting_user))
    .route("/{user_id}", web::patch().to_async(update_user))
    .route("/{user_id}", web::get().to_async(get_user_by_id))
}