use diesel::prelude::*;
use actix_web::{ web, HttpResponse, dev, Error };
// use futures::future::{ Future, ok as fut_ok };
use uuid::Uuid;

use crate::models::org::{ NewOrg, Org, IsertableNewOrg };
use crate::models::user::{ User, Auth0UserId };
use crate::db::Pool;
use crate::utils::general::query_to_response;

use crate::schema::{ orgs, users };

use orgs::{ table as OrgTable, dsl as OrgQuery };
use users::{ table as UserTable, dsl as UserQuery };

#[derive(Deserialize)]
pub struct UserIdPath {
  user_id: Uuid
}

#[derive(Deserialize)]
pub struct OrgIdPath {
  org_id: Uuid
}

async fn get_org(
  path: web::Path<OrgIdPath>,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  query_to_response(move || -> diesel::QueryResult<Org> {
    let conn: &PgConnection = &pool.get().unwrap();

    OrgTable
      .find(path.org_id)
      .get_result::<Org>(conn)
  })
  .await
}

// Given a vec of org ids, return orgs.
async fn get_user_orgs(
  path: web::Path<UserIdPath>,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  query_to_response(move || -> diesel::QueryResult<Vec<Org>> {
    let conn: &PgConnection = &pool.get().unwrap();

    let user = UserTable
      .filter(UserQuery::id.eq(path.user_id))
      .get_result::<User>(conn).unwrap();

    OrgTable
      .filter(OrgQuery::id.eq_any(user.orgs))
      .get_results::<Org>(conn)
  }).await
}

async fn create_org(
  auth0_user_id: Auth0UserId,
  new_org: NewOrg,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  query_to_response(move || -> Result<Org, diesel::result::Error> {
    let conn: &PgConnection = &pool.get().unwrap();

    let user = UserTable.filter(UserQuery::auth0id.eq(&auth0_user_id.id))
      .get_result::<User>(conn).ok().unwrap();

    let insertable_new_org = IsertableNewOrg {
      name: new_org.name,
      created_by: user.id,
      updated_by: user.id,
    };

    diesel::insert_into(OrgTable)
      .values(insertable_new_org)
      .get_result::<Org>(conn)
  }).await
}

pub fn get_org_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/orgs")
    .route("/org", web::post().to(create_org))
    .route("/{org_id}", web::get().to(get_org))
    .route("/user/{user_id}", web::get().to(get_user_orgs))
}