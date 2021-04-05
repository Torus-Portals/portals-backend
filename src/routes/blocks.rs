// use diesel::prelude::*;
// use actix_web::{ web, HttpResponse, dev, Error };
// use uuid::Uuid;

// use crate::db::Pool;
// use crate::utils::general::query_to_response;

// use crate::models::user::{ User, Auth0UserId };

// use crate::models::block::{ Block, NewBlockPayload, NewBlock };

// use crate::schema::{ users, blocks };
// use users::{ table as UserTable, dsl as UserQuery };
// use blocks::{ table as BlockTable, dsl as BlockQuery };

// #[derive(Deserialize)]
// pub struct PortalId {
//   portal_id: Uuid
// }

// async fn create_block(
//   auth0_user_id: Auth0UserId,
//   new_block_payload: NewBlockPayload,
//   pool: web::Data<Pool>
// ) -> Result<HttpResponse, Error> {
//   query_to_response(move || -> diesel::QueryResult<Block> {
//     let conn: &PgConnection = &pool.get().unwrap();

//     let user = UserTable.filter(UserQuery::auth0id.eq(&auth0_user_id.id))
//       .get_result::<User>(conn).ok().unwrap();

//     let new_block = NewBlock {
//       block_type: new_block_payload.block_type,
//       portal_id: new_block_payload.portal_id,
//       portal_view_id: new_block_payload.portal_view_id,
//       egress: new_block_payload.egress,
//       bbox: new_block_payload.bbox,
//       data: new_block_payload.data,
//       created_by: user.id,
//       updated_by: user.id,
//     };

//     diesel::insert_into(BlockTable)
//       .values(new_block)
//       .get_result::<Block>(conn)
//   })
//   .await
// }

// async fn get_portal_blocks(
//   path: web::Path<PortalId>,
//   pool: web::Data<Pool>
// ) -> Result<HttpResponse, Error> {
//   query_to_response(move || -> diesel::QueryResult<Vec<Block>> {
//     let conn: &PgConnection = &pool.get().unwrap();

//     BlockTable.filter(BlockQuery::portal_id.eq(path.portal_id))
//     .get_results::<Block>(conn)
//   }).await
// }

// pub fn get_block_routes() -> impl dev::HttpServiceFactory + 'static {
//   web::scope("/blocks")
//     .route("/block", web::post().to(create_block))
//     .route("/portal/{portal_id}", web::get().to(get_portal_blocks))
// }