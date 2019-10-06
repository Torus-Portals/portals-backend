use diesel::prelude::*;
use actix_web::{ web, HttpResponse, dev, Error };
use futures::future::{ Future, ok as fut_ok };
use uuid::Uuid;

use crate::models::portal::{ Portal, NewPortalPayload, NewPortal };
use crate::models::portalview::{ PortalView, NewPortalView };
use crate::models::user::{ User, Auth0UserId };
use crate::db::Pool;

use crate::schema::{ users, portals, portalviews };
use users::{ table as UserTable, dsl as UserQuery };
use portals::{ table as PortalTable, dsl as PortalQuery };
use portalviews::{ table as PortalViewTable, dsl as PortalViewQuery };

#[derive(Deserialize)]
pub struct PortalId {
  portal_id: Uuid
}

pub fn get_portal(
  path: web::Path<PortalId>,
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  web::block(move || -> diesel::QueryResult<Portal> {
    let conn: &PgConnection = &pool.get().unwrap();

    PortalTable.filter(PortalQuery::id.eq(path.portal_id))
    .get_result::<Portal>(conn)
  })
  .then(|res| match res {
    Ok(portal) => fut_ok(HttpResponse::Ok().json(portal)),
    Err(_) => fut_ok(HttpResponse::InternalServerError().into())
  })
}

pub fn create_portal(
  auth0_user_id: Auth0UserId,
  new_portal_payload: NewPortalPayload,
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  web::block(move || -> Result<Portal, diesel::result::Error> {
    let conn: &PgConnection = &pool.get().unwrap();

    // Try to find user by auth0 id from jwt
    let user = UserTable.filter(UserQuery::auth0id.eq(&auth0_user_id.id))
      .get_result::<User>(conn).ok().unwrap();

    // crate portal
    let new_portal = NewPortal {
      org: new_portal_payload.org,
      name: new_portal_payload.name,
      created_by: user.id,
      updated_by: user.id,
      owners: vec![user.id],
    };

    let created_portal = diesel::insert_into(PortalTable)
      .values(new_portal)
      .get_result::<Portal>(conn)?;

    // Create default owner portal view
    let default_owner_portal_view = NewPortalView {
      portal_id: created_portal.id,
      name: String::from(""),
      egress: String::from("owner"),
      access: String::from("public"),
      created_by: user.id,
      updated_by: user.id,
    };

    diesel::insert_into(PortalViewTable)
      .values(default_owner_portal_view)
      .execute(conn)?;

    // Create default vendor portal view
    let default_vendor_portal_view = NewPortalView {
      portal_id: created_portal.id,
      name: String::from(""),
      egress: String::from("vendor"),
      access: String::from("public"),
      created_by: user.id,
      updated_by: user.id,
    };

    diesel::insert_into(PortalViewTable)
      .values(default_vendor_portal_view)
      .execute(conn)?;

    Ok(created_portal)
  })
  .then(|res| match res {
    Ok(portal) => fut_ok(HttpResponse::Ok().json(portal)),
    Err(err) => {
      println!("create_portal err: {:#?}", err);
      fut_ok(HttpResponse::InternalServerError().into())
    }
  })
}

pub fn get_portal_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/portals")
    .route("/portal", web::post().to_async(create_portal))
    .route("/{portal_id}", web::get().to_async(get_portal))
}