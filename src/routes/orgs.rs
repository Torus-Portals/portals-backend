use diesel::prelude::*;
use actix_web::{ web, HttpResponse, dev, Error };
use futures::future::{ Future, ok as fut_ok };

use crate::models::org::{ NewOrg, Org };
use crate::models::user::{ User };
use crate::db::Pool;

use crate::schema::{ orgs, users };

use orgs::{ table as OrgTable, dsl as OrgQuery };
use users::{ table as UserTable };

#[derive(Deserialize)]
pub struct UserIdPath {
  user_id: i32
}

#[derive(Deserialize)]
pub struct OrgIdPath {
  org_id: i32
}

fn get_org(
  path: web::Path<OrgIdPath>,
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  web::block(move || -> diesel::QueryResult<Org> {
    let conn: &PgConnection = &pool.get().unwrap();

    OrgTable
      .find(path.org_id)
      .get_result::<Org>(conn)
  })
  .then(|res| match res {
    Ok(org) => fut_ok(HttpResponse::Ok().json(org)),
    Err(_) => fut_ok(HttpResponse::InternalServerError().into())
  })
}


// Given a vec of org ids, return orgs.
fn get_user_orgs(
  path: web::Path<UserIdPath>,
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  web::block(move || -> diesel::QueryResult<Vec<Org>> {
    let conn: &PgConnection = &pool.get().unwrap();

    let user = UserTable
      .find(path.user_id)
      .get_result::<User>(conn).unwrap();

    OrgTable
      .filter(OrgQuery::id.eq_any(user.orgs))
      .get_results::<Org>(conn)
  })
  .then(|res| match res {
    Ok(orgs) => fut_ok(HttpResponse::Ok().json(orgs)),
    Err(_) => fut_ok(HttpResponse::InternalServerError().into())
  })
}

fn create_org(
  new_org: NewOrg,
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  web::block(move || -> Result<Org, diesel::result::Error> {
    let conn: &PgConnection = &pool.get().unwrap();

    diesel::insert_into(OrgTable)
      .values(new_org)
      .get_result::<Org>(conn)
  })
  .then(|res| match res {
    Ok(org) => fut_ok(HttpResponse::Ok().json(org)),
    Err(_) => fut_ok(HttpResponse::InternalServerError().into())
  })
}


pub fn get_org_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/orgs")
    .route("", web::post().to_async(create_org))
    .route("/{org_id}", web::get().to_async(get_org))
    .route("/user/{user_id}", web::get().to_async(get_user_orgs))
}