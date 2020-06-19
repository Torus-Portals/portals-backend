use diesel::prelude::*;
use actix_web::{ web, HttpResponse, dev, Error };
use actix_rt::Runtime;
use uuid::Uuid;

use std::convert::From;

use crate::utils::general::query_to_response;
use crate::utils::async_block::{ async_block, BlockError, BlockResponse, BlockResult };

use crate::models::user::{ User, NewUser, UpdateUser, InvitedUser, Auth0UserId };
use crate::models::portal::{ Portal };
use crate::db::Pool;

use crate::schema::{ users, portals };
use users::{ table as UserTable, dsl as UserQuery };
use portals::{ table as PortalTable, dsl as PortalQuery };

use crate::services::auth0_service::{ get_auth0_user, get_auth0_token };
use crate::services::email_service::{ send_invitation_email };

use crate::queries::user_queries::{ get_user, get_user_by_email };


#[derive(Deserialize)]
pub struct UserIdPath {
  user_id: Uuid
}

async fn create_user(
  auth0_user_id: Auth0UserId,
  new_user: NewUser,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  Ok(web::block(move || -> Result<User, diesel::result::Error> {
    let conn: &PgConnection = &pool.get().unwrap();

    // Try to find user by auth0 id from jwt

    let user = get_user(auth0_user_id, conn)?;

    let new_user_with_created_by = NewUser {
      created_by: user.id,
      ..new_user
    };

    diesel::insert_into(UserTable)
      .values(new_user_with_created_by)
      .get_result::<User>(conn)
  })
  .await
  .map(|user| HttpResponse::Ok().json(user))
  .map_err(|_| HttpResponse::InternalServerError())?)
}

async fn get_requesting_user(
  auth0_user_id: Auth0UserId,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  let pool1 = pool.clone();

  println!("Getting requesting_user");

  Ok(web::block(move || -> Result<User, diesel::result::Error> {
    let conn: &PgConnection = &pool1.get().unwrap();

    // Try to find user by auth0 id from jwt
    let res = UserTable.filter(UserQuery::auth0id.eq(&auth0_user_id.id))
      .get_result::<User>(conn);

    match res {
        // If found, return
        Ok(user) => Ok(user),
        Err(err) => {
          // If user isn't found in db, see if user is in auth0
          match get_auth0_user(&auth0_user_id.id) {
            Ok(auth0_user) => {
              let new_user = NewUser {
                auth0id: auth0_user.user_id,
                name: auth0_user.name,
                nickname: auth0_user.nickname,
                email: auth0_user.email,
                status: String::from("active"),
                created_by: Uuid::parse_str(&"11111111-2222-3333-4444-555555555555").ok().unwrap(),
                updated_by: Uuid::parse_str(&"11111111-2222-3333-4444-555555555555").ok().unwrap(),
              };

              diesel::insert_into(UserTable)
                .values(new_user)
                .get_result::<User>(conn)
            },
            Err(err2) => {
              println!("err2: {:#?}", err2);
              Err(err)
            }
          }
        }
      }
  })
  .await
  .map(|user| HttpResponse::Ok().json(user))
  .map_err(|_| HttpResponse::InternalServerError())?)
}

async fn update_user(
  auth0_user_id: Auth0UserId,
  path: web::Path<UserIdPath>,
  updated_user: UpdateUser,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  Ok(web::block(move || -> Result<User, diesel::result::Error> {
    let conn: &PgConnection = &pool.get().unwrap();

    let res = UserTable.filter(UserQuery::auth0id.eq(&auth0_user_id.id))
      .get_result::<User>(conn).ok().unwrap();

    let updated_user_with_updated_by = UpdateUser {
      updated_by: Some(res.id),
      ..updated_user
    };

    diesel::update(UserTable.filter(UserQuery::id.eq(path.user_id)))
      .set(updated_user_with_updated_by)
      .get_result::<User>(conn)
  })
  .await
  .map(|user| HttpResponse::Ok().json(user))
  .map_err(|_| HttpResponse::InternalServerError())?)
}

async fn get_user_by_id(
  path: web::Path<UserIdPath>,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  Ok(web::block(move || -> diesel::QueryResult<User> {
    let conn: &PgConnection = &pool.get().unwrap();

    UserTable
      .filter(UserQuery::id.eq(path.user_id))
      .get_result::<User>(conn)
  })
  .await
  .map(|user| HttpResponse::Ok().json(user))
  .map_err(|_| HttpResponse::InternalServerError())?)
}

#[derive(Debug, Serialize)]
struct PortalAndUser {
  portal: Portal,
  user: User,
}

async fn invite_user_to_portal(
  invited_user: InvitedUser,
  pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
  async_block(move || -> BlockResult<PortalAndUser> {
    let conn: &PgConnection = &pool.get().unwrap();

    let user = match get_user_by_email(&invited_user.email, conn).ok() {
      Some(user) => user,
      None => {
        let new_user = NewUser {
          auth0id: String::from(""),
          name: String::from(""),
          nickname: String::from(""),
          email: invited_user.email.to_string(),
          status: String::from("invited"),
          created_by: Uuid::parse_str(&"11111111-2222-3333-4444-555555555555").ok().unwrap(),
          updated_by: Uuid::parse_str(&"11111111-2222-3333-4444-555555555555").ok().unwrap(),
        };

        diesel::insert_into(UserTable)
          .values(new_user)
          .get_result::<User>(conn)?
      }
    };

    let mut portal = PortalTable.filter(PortalQuery::id.eq(invited_user.portal_id))
      .get_result::<Portal>(conn)?;

      
      // Make sure user is not currently in the portal.
    if portal.owners.contains(&user.id) || portal.vendors.contains(&user.id) {
      return Err(BlockError::BadRequest("User already belongs to this portal".into()))
    }
    
    // Update portal owners/vendors with additional invited user. 
    match invited_user.egress.as_str() {
      "owner" => {
        if !portal.owners.contains(&user.id) {
          portal.owners.push(user.id);
        }
      },
      "vendor" => {
        if !portal.vendors.contains(&user.id) {
          portal.vendors.push(user.id);
        }
      },
      _ => unreachable!()
    }

    let portal = diesel::update(PortalTable.filter(PortalQuery::id.eq(invited_user.portal_id)))
      .set((PortalQuery::vendors.eq(portal.vendors), PortalQuery::owners.eq(portal.owners)))
      .get_result::<Portal>(conn)?;

    // Add the portals's org to the user's orgs, if the user is not a member of the portal's org already.
    let user = {
      if !user.orgs.contains(&portal.org) {
        let mut new_user_orgs = user.orgs;
  
        new_user_orgs.push(portal.org);
  
        diesel::update(UserTable.filter(UserQuery::id.eq(user.id)))
          .set(UserQuery::orgs.eq(new_user_orgs))
          .get_result::<User>(conn)?
      } else {
        user
      }
    };

    let mut rt = Runtime::new().unwrap();

    rt.block_on(send_invitation_email())?;

    // Email user an invite!
    // let thing = async {
    //   send_invitation_email().await;
    // };

    // Runtime.block_on(send_invitation_email());


    Ok(BlockResponse::JSON(PortalAndUser {
      portal,
      user
    }))
  })
  .await
}


// TODO: This is where I've left off. Try sending an email!!
// async fn test_send_email() -> Result<HttpResponse, Error> {
//  match send_invitation_email().await {
//    Ok
//  }

//  Ok(HttpResponse::Ok().finish())
// }

pub fn get_user_routes() -> impl dev::HttpServiceFactory + 'static {
  web::scope("/users")
    .route("", web::post().to(create_user))
    .route("/user", web::get().to(get_requesting_user))
    .route("/user/invite", web::post().to(invite_user_to_portal))
    .route("/{user_id}", web::patch().to(update_user))
    .route("/{user_id}", web::get().to(get_user_by_id))
    // .route("/email", web::get().to(test_send_email))
}