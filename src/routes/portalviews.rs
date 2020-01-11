use diesel::prelude::*;
use actix_web::{ web, HttpResponse, dev, Error };
use uuid::Uuid;

use crate::db::Pool;
use crate::utils::general::query_to_response;

use crate::models::portalview::{ PortalView };

use crate::schema::{ portalviews };
use portalviews::{ table as PortalViewTable, dsl as PortalViewQuery };

#[derive(Deserialize)]
pub struct PortalId {
  portal_id: Uuid
}

async fn get_portal_portalviews(
  path: web::Path<PortalId>,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  query_to_response(move || -> diesel::QueryResult<Vec<PortalView>> {
    let conn: &PgConnection = &pool.get().unwrap();

    PortalViewTable.filter(PortalViewQuery::portal_id.eq(path.portal_id))
    .get_results::<PortalView>(conn)
  }).await
}

pub fn get_portalview_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/portalviews")
    .route("/portal/{portal_id}", web::get().to(get_portal_portalviews))
}