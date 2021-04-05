// use diesel::prelude::*;
// use actix_web::{ web, HttpRequest, HttpResponse, dev, Error };
// use uuid::Uuid;
// use serde_qs as qs;

// use crate::db::Pool;
// use crate::utils::general::query_to_response;

// use crate::models::user::{ Auth0UserId };
// use crate::models::cell::{ Cell, NewCellsPayload, NewCell, UpdateCell };

// use crate::schema::{ cells };
// use cells::{ table as CellTable, dsl as CellQuery };

// use crate::queries::user_queries::{ get_user };

// #[derive(Deserialize)]
// pub struct CellIdPath {
//   cell_id: Uuid
// }

// async fn create_cells(
//   auth0_user_id: Auth0UserId,
//   new_cells_payload: NewCellsPayload,
//   pool: web::Data<Pool>
// ) -> Result<HttpResponse, Error> {
//   query_to_response(move || -> diesel::QueryResult<Vec<Cell>> {
//     let conn: &PgConnection = &pool.get().unwrap();

//     let user = get_user(auth0_user_id, conn)?;

//     let new_cells: Vec<NewCell> = new_cells_payload.0.into_iter().map(|cell_payload| {
//       NewCell {
//         portal_id: cell_payload.portal_id,
//         cell_type: cell_payload.cell_type,
//         dimensions: cell_payload.dimensions,
//         data: cell_payload.data,
//         created_by: user.id,
//         updated_by: user.id,
//       }
//     }).collect();

//     diesel::insert_into(CellTable)
//       .values(new_cells)
//       .get_results::<Cell>(conn)
//   })
//   .await
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// struct DimensionsQuery {
//   dimensions: Vec<Uuid>,
// }

// async fn get_cells_by_dimensions(
//   // dimensions: QsQuery<DimensionsQuery>,
//   pool: web::Data<Pool>,
//   req: HttpRequest
// ) -> Result<HttpResponse, Error> {
//   // TODO: this will crash the server if given non Uuid params.
//   //       Would be nice to have some better way to check query strings.
//   let query = qs::from_str::<DimensionsQuery>(req.query_string());

//   match query {
//     Ok(q) => {
//       query_to_response(move || -> diesel::QueryResult<Vec<Cell>> {
//         let conn: &PgConnection = &pool.get().unwrap();
    
//         CellTable.filter(CellQuery::dimensions.is_contained_by(q.dimensions))
//         .get_results::<Cell>(conn)
//       })
//       .await
//     }
//     Err(err) => {
//       dbg!(err);
//       Ok(HttpResponse::Ok().finish().into_body()) 
//     }
//   }
// }

// async fn update_cell(
//   auth0_user_id: Auth0UserId,
//   path: web::Path<CellIdPath>,
//   pool: web::Data<Pool>,
//   update_cell: UpdateCell
// ) -> Result<HttpResponse, Error> {
//   query_to_response(move || -> diesel::QueryResult<Cell> {
//     let conn: &PgConnection = &pool.get().unwrap();
//     let user = get_user(auth0_user_id, conn)?;

//     let updated_cell_with_updated_by = UpdateCell {
//       updated_by: Some(user.id),
//       ..update_cell
//     };

//     diesel::update(CellTable.filter(CellQuery::id.eq(path.cell_id)))
//       .set(updated_cell_with_updated_by)
//       .get_result::<Cell>(conn)
//   })
//   .await
// }

// pub fn get_cell_routes() -> impl dev::HttpServiceFactory + 'static {
//   web::scope("/cells")
//     .route("/dimensions", web::get().to(get_cells_by_dimensions))
//     .route("/{cell_id}", web::patch().to(update_cell))
//     .route("", web::post().to(create_cells))
// }
