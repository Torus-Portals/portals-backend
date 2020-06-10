use diesel::prelude::*;

use crate::models::user::{ User, Auth0UserId };

use crate::schema::{ users };
use users::{ table as UserTable, dsl as UserQuery };

pub fn get_user(auth0_user_id: Auth0UserId, conn: &PgConnection) -> diesel::QueryResult<User> {
  UserTable.filter(UserQuery::auth0id.eq(&auth0_user_id.id))
      .get_result::<User>(conn)
}

pub fn get_user_by_email(email: &str, conn: &PgConnection) -> diesel::QueryResult<User> {
  UserTable.filter(UserQuery::email.eq(&email))
    .get_result::<User>(conn)
}