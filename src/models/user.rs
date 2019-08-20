use crate::schema::users;

#[derive(Serialize, Queryable)]
pub struct User {
  pub id: i32,
  pub username: String,
  pub firstname: String,
  pub lastname: String,
  pub email: String,
  pub email_confirmed: bool,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
  pub id: i32,
  pub username: &'a str,
  pub firstname: &'a str,
  pub lastname: &'a str,
  pub email: &'a str,
}