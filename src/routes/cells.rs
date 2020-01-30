use diesel::prelude::*;
use actix_web::{ web, HttpResponse, dev, Error };

use crate::db::Pool;
use crate::utils::general::query_to_response;
use crate::models::user::{ User, Auth0UserId };

use crate::models::cell::{ Cell, NewCellsPayload, NewCell };

use crate::schema::{ users, cells };
use users::{ table as UserTable, dsl as UserQuery };
use cells::{ table as CellTable };

async fn create_cells(
  auth0_user_id: Auth0UserId,
  new_cells_payload: NewCellsPayload,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  query_to_response(move || -> diesel::QueryResult<Vec<Cell>> {
    let conn: &PgConnection = &pool.get().unwrap();

    let user = UserTable.filter(UserQuery::auth0id.eq(&auth0_user_id.id))
      .get_result::<User>(conn).ok().unwrap();

    let new_cells: Vec<NewCell> = new_cells_payload.0.into_iter().map(|cell_payload| {
      NewCell {
        portal_id: cell_payload.portal_id,
        dimensions: cell_payload.dimensions,
        data: cell_payload.data,
        created_by: user.id,
        updated_by: user.id,
      }
    }).collect();

    diesel::insert_into(CellTable)
      .values(new_cells)
      .get_results::<Cell>(conn)
  })
  .await
}

pub fn get_cell_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/cells")
    .route("", web::post().to(create_cells))
}