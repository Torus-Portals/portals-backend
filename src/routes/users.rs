use diesel::prelude::*;
use actix_web::{ web, HttpResponse, dev, Error };
use uuid::Uuid;

use crate::models::user::{ User, NewUser, UpdateUser, Auth0UserId };
use crate::db::Pool;

use crate::schema;

use crate::services::auth0_service::get_auth0_user;

use crate::queries::user_queries::{ get_user };

#[derive(Deserialize)]
pub struct UserIdPath {
  user_id: Uuid
}

async fn create_user(
  auth0_user_id: Auth0UserId,
  new_user: NewUser,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  Ok(web::block(move || -> Result<User, diesel::result::Error> {
    use schema::users::dsl::*;

    let conn: &PgConnection = &pool.get().unwrap();

    // Try to find user by auth0 id from jwt

    let user = get_user(auth0_user_id, conn)?;

    let new_user_with_created_by = NewUser {
      created_by: user.id,
      ..new_user
    };

    diesel::insert_into(users)
      .values(new_user_with_created_by)
      .get_result::<User>(conn)
  })
  .await
  .map(|user| HttpResponse::Ok().json(user))
  .map_err(|_| HttpResponse::InternalServerError())?)
}

async fn get_requesting_user(
  auth0_user_id: Auth0UserId,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  let pool1 = pool.clone();

  Ok(web::block(move || -> Result<User, diesel::result::Error> {
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
                created_by: Uuid::parse_str(&"11111111-2222-3333-4444-555555555555").ok().unwrap(),
                updated_by: Uuid::parse_str(&"11111111-2222-3333-4444-555555555555").ok().unwrap(),
              };

              diesel::insert_into(users)
                .values(new_user)
                .get_result::<User>(conn)
            },
            Err(err2) => {
              println!("err2: {:#?}", err2);
              Err(err)
            }
          }
        }
      }
  })
  .await
  .map(|user| HttpResponse::Ok().json(user))
  .map_err(|_| HttpResponse::InternalServerError())?)
}

async fn update_user(
  auth0_user_id: Auth0UserId,
  path: web::Path<UserIdPath>,
  updated_user: UpdateUser,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  Ok(web::block(move || -> Result<User, diesel::result::Error> {
    use schema::users::dsl::*;

    let conn: &PgConnection = &pool.get().unwrap();

    let res = users.filter(auth0id.eq(&auth0_user_id.id))
      .get_result::<User>(conn).ok().unwrap();

    let updated_user_with_updated_by = UpdateUser {
      updated_by: Some(res.id),
      ..updated_user
    };

    diesel::update(users.filter(id.eq(path.user_id)))
      .set(updated_user_with_updated_by)
      .get_result::<User>(conn)
  })
  .await
  .map(|user| HttpResponse::Ok().json(user))
  .map_err(|_| HttpResponse::InternalServerError())?)
}

async fn get_user_by_id(
  path: web::Path<UserIdPath>,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  Ok(web::block(move || -> diesel::QueryResult<User> {
    let conn: &PgConnection = &pool.get().unwrap();

    schema::users::table
      .filter(schema::users::dsl::id.eq(path.user_id))
      .get_result::<User>(conn)
  })
  .await
  .map(|user| HttpResponse::Ok().json(user))
  .map_err(|_| HttpResponse::InternalServerError())?)
}


pub fn get_user_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/users")
    .route("", web::post().to(create_user))
    .route("/user", web::get().to(get_requesting_user))
    .route("/{user_id}", web::patch().to(update_user))
    .route("/{user_id}", web::get().to(get_user_by_id))
}