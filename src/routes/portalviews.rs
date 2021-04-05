// use diesel::prelude::*;
// use actix_web::{ web, HttpResponse, dev, Error };
// use uuid::Uuid;

// use crate::db::Pool;
// use crate::utils::general::query_to_response;

// use crate::models::user::{ Auth0UserId };
// use crate::models::portalview::{ PortalView, NewPortalView, NewPortalViewPayload };

// use crate::schema::{ portalviews };
// use portalviews::{ table as PortalViewTable, dsl as PortalViewQuery };

// use crate::queries::user_queries::{ get_user };

// #[derive(Deserialize)]
// pub struct PortalId {
//   portal_id: Uuid
// }

// async fn get_portal_portalviews(
//   path: web::Path<PortalId>,
//   pool: web::Data<Pool>
// ) -> Result<HttpResponse, Error> {
//   query_to_response(move || -> diesel::QueryResult<Vec<PortalView>> {
//     let conn: &PgConnection = &pool.get().unwrap();

//     PortalViewTable.filter(PortalViewQuery::portal_id.eq(path.portal_id))
//     .get_results::<PortalView>(conn)
//   }).await
// }

// async fn create_portalview(
//   auth0_user_id: Auth0UserId,
//   new_portalview_payload: NewPortalViewPayload,
//   pool: web::Data<Pool>
// ) -> Result<HttpResponse, Error> {
//   query_to_response(move || -> diesel::QueryResult<PortalView> {
//     let conn: &PgConnection = &pool.get().unwrap();

//     let user = get_user(auth0_user_id, conn)?;

//     let new_portalview = NewPortalView {
//       portal_id: new_portalview_payload.portal_id,
//       name: new_portalview_payload.name,
//       egress: new_portalview_payload.egress,
//       access: new_portalview_payload.access,
//       created_by: user.id,
//       updated_by: user.id,
//     };

//     diesel::insert_into(PortalViewTable)
//       .values(new_portalview)
//       .get_result::<PortalView>(conn)
//   }).await
// }

// pub fn get_portalview_routes() -> impl dev::HttpServiceFactory + 'static {
//   web::scope("/portalviews")
//     .route("/portal/{portal_id}", web::get().to(get_portal_portalviews))
//     .route("/portalview", web::post().to(create_portalview))
// }