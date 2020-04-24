use diesel::prelude::*;
// use diesel::dsl::*;
use actix_web::{ web, HttpResponse, dev, Error };
use uuid::Uuid;

use crate::models::portal::{ Portal, NewPortalPayload, NewPortal, UpdatePortal };
use crate::models::portalview::{ /* PortalView, */ NewPortalView };
use crate::models::user::{ User, Auth0UserId };
use crate::db::Pool;
use crate::utils::general::query_to_response;

use crate::schema::{ users, portals, portalviews };
use users::{ table as UserTable, dsl as UserQuery };
use portals::{ table as PortalTable, dsl as PortalQuery };
use portalviews::{ table as PortalViewTable /*, dsl as PortalViewQuery*/ };

#[derive(Deserialize)]
pub struct PortalId {
  portal_id: Uuid
}

async fn get_portal(
  path: web::Path<PortalId>,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  query_to_response(move || -> diesel::QueryResult<Portal> {
    let conn: &PgConnection = &pool.get().unwrap();

    PortalTable.filter(PortalQuery::id.eq(path.portal_id))
    .get_result::<Portal>(conn)
  }).await
}

#[derive(Serialize)]
struct PortalsByEgress {
  owner: Vec<Portal>,
  vendor: Vec<Portal>
}

async fn get_portals(
  auth0_user_id: Auth0UserId,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  query_to_response(move || -> diesel::QueryResult<PortalsByEgress> {    
    let conn: &PgConnection = &pool.get().unwrap();

    let user = UserTable.filter(UserQuery::auth0id.eq(&auth0_user_id.id))
      .get_result::<User>(conn).ok().unwrap();

    let portals_with_user_as_owner = PortalTable.filter(PortalQuery::owners.contains(vec![user.id]))
    .get_results::<Portal>(conn).ok().unwrap();

    let portals_with_user_as_vendor = PortalTable.filter(PortalQuery::vendors.contains(vec![user.id]))
    .get_results::<Portal>(conn).ok().unwrap();

    Ok(PortalsByEgress {
      owner: portals_with_user_as_owner,
      vendor: portals_with_user_as_vendor
    })
  }).await
}

async fn create_portal(
  auth0_user_id: Auth0UserId,
  new_portal_payload: NewPortalPayload,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  query_to_response(move || -> Result<Portal, diesel::result::Error> {
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
      name: String::from("Default Owner View"),
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
      name: String::from("Default Vendor View"),
      egress: String::from("vendor"),
      access: String::from("public"),
      created_by: user.id,
      updated_by: user.id,
    };

    diesel::insert_into(PortalViewTable)
      .values(default_vendor_portal_view)
      .execute(conn)?;

    Ok(created_portal)
  }).await
}

async fn update_portal(
  auth0_user_id: Auth0UserId,
  path: web::Path<PortalId>,
  updated_portal: UpdatePortal,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  query_to_response(move || -> Result<Portal, diesel::result::Error> {
    let conn: &PgConnection = &pool.get().unwrap();

    let res = UserTable.filter(UserQuery::auth0id.eq(&auth0_user_id.id))
      .get_result::<User>(conn).ok().unwrap();

    let updated_portal_with_updated_by = UpdatePortal {
      updated_by: Some(res.id),
      ..updated_portal
    };

    diesel::update(PortalTable.filter(PortalQuery::id.eq(path.portal_id)))
      .set(updated_portal_with_updated_by)
      .get_result(conn)
  }).await
}

pub fn get_portal_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/portals")
    .route("", web::get().to(get_portals))
    .route("/portal", web::post().to(create_portal))
    .route("/{portal_id}", web::get().to(get_portal))
    .route("/{portal_id}", web::patch().to(update_portal))
}