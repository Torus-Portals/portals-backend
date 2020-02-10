use diesel::prelude::*;
use actix_web::{ web, HttpResponse, dev, Error };
use uuid::Uuid;

use crate::db::Pool;
use crate::utils::general::query_to_response;

use crate::models::user::{ Auth0UserId };
use crate::models::dimension::{
  Dimension,
  NewDimension,
  NewDimensionPayload,
  NewDimensionsPayload
};

use crate::schema::{ dimensions };

use dimensions::{ table as DimensionTable, dsl as DimensionQuery };

use crate::queries::user_queries::{ get_user };


#[derive(Deserialize)]
struct PortalId {
  portal_id: Uuid
}

async fn get_portal_dimensions(
  path: web::Path<PortalId>,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  query_to_response(move || -> diesel::QueryResult<Vec<Dimension>> {
    let conn: &PgConnection = &pool.get().unwrap();

    DimensionTable.filter(DimensionQuery::portal_id.eq(path.portal_id))
      .get_results::<Dimension>(conn)
  })
  .await
}

async fn create_dimension(
  auth0_user_id: Auth0UserId,
  new_dimension_payload: NewDimensionPayload,
  pool: web::Data<Pool> 
) -> Result<HttpResponse, Error> {
  query_to_response(move || -> diesel::QueryResult<Dimension> {
    let conn: &PgConnection = &pool.get().unwrap();

    let user = get_user(auth0_user_id, conn)?;

    let new_dimension = NewDimension {
      portal_id: new_dimension_payload.portal_id,
      name: new_dimension_payload.name,
      dimension_type: new_dimension_payload.dimension_type,
      meta: new_dimension_payload.meta,
      created_by: user.id,
      updated_by: user.id
    };

    diesel::insert_into(DimensionTable)
      .values(new_dimension)
      .get_result::<Dimension>(conn)
  })
  .await
}

async fn create_dimensions(
  auth0_user_id: Auth0UserId,
  new_dimensions_payload: NewDimensionsPayload,
  pool: web::Data<Pool> 
) -> Result<HttpResponse, Error> {
  query_to_response(move || -> diesel::QueryResult<Vec<Dimension>> {

    let conn: &PgConnection = &pool.get().unwrap();

    let user = get_user(auth0_user_id, conn)?;

    let new_dimensions: Vec<NewDimension> = new_dimensions_payload.0.into_iter().map(|dimension| {
      NewDimension {
        portal_id: dimension.portal_id,
        name: dimension.name,
        dimension_type: dimension.dimension_type,
        meta: dimension.meta,
        created_by: user.id,
        updated_by: user.id
      }
    }).collect();

    diesel::insert_into(DimensionTable)
      .values(new_dimensions)
      .get_results::<Dimension>(conn)
  })
  .await
}

pub fn get_dimension_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/dimensions")
    .route("", web::post().to(create_dimensions))
    .route("/dimension", web::post().to(create_dimension))
    .route("/portal/{portal_id}", web::get().to(get_portal_dimensions))
}