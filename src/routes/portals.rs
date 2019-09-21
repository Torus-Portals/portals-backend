use diesel::prelude::*;
use actix_web::{ web, HttpResponse, dev, Error };
use futures::future::{ Future, ok as fut_ok };
// use uuid::Uuid;

use crate::models::portal::{ Portal, NewPortal };
use crate::models::user::{ User, Auth0UserId };
use crate::db::Pool;

use crate::schema::{ users, portals };
use users::{ table as UserTable, dsl as UserQuery };
use portals::{ table as PortalTable };

// pub fn get_org_portals() -> impl Future<Item = HttpResponse, Error = Error> {

// }

pub fn create_portal(
  auth0_user_id: Auth0UserId,
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  web::block(move || -> Result<Portal, diesel::result::Error> {
    let conn: &PgConnection = &pool.get().unwrap();

    // Try to find user by auth0 id from jwt
    let user = UserTable.filter(UserQuery::auth0id.eq(&auth0_user_id.id))
      .get_result::<User>(conn).ok().unwrap();

    let new_portal_with_created_by = NewPortal {
      created_by: user.id,
      updated_by: user.id,
      owners: vec![user.id],
    };

    diesel::insert_into(PortalTable)
      .values(new_portal_with_created_by)
      .get_result::<Portal>(conn)
  })
  .then(|res| match res {
    Ok(portal) => fut_ok(HttpResponse::Ok().json(portal)),
    Err(_) => fut_ok(HttpResponse::InternalServerError().into())
  })
}

pub fn get_portal_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/portals")
    .route("/portal", web::post().to_async(create_portal))
}