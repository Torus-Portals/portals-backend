use diesel::prelude::*;
use actix_web::{ web, HttpResponse, dev, Error };
use futures::future::{ Future, ok as fut_ok };
use uuid::Uuid;

use crate::db::Pool;

use crate::models::portalview::{ PortalView };

use crate::schema::{ portalviews };
use portalviews::{ table as PortalViewTable, dsl as PortalViewQuery };

#[derive(Deserialize)]
pub struct PortalId {
  portal_id: Uuid
}

pub fn get_portal_portalviews(
  path: web::Path<PortalId>,
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  web::block(move || -> diesel::QueryResult<Vec<PortalView>> {
    let conn: &PgConnection = &pool.get().unwrap();

    PortalViewTable.filter(PortalViewQuery::portal_id.eq(path.portal_id))
    .get_results::<PortalView>(conn)
  })
  .then(|res| match res {
    Ok(portalviews) => fut_ok(HttpResponse::Ok().json(portalviews)),
    Err(_) => fut_ok(HttpResponse::InternalServerError().into())
  })
}

pub fn get_portalview_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/portalviews")
    .route("/portal/{portal_id}", web::get().to_async(get_portal_portalviews))
}