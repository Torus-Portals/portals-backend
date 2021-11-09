use async_trait::async_trait;
use dataloader::non_cached::Loader;
use dataloader::BatchFn;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::graphql::schema::user::User;
use crate::services::db::user_service::get_project_users;

pub struct UserBatcher {
  pool: PgPool,
}

#[async_trait]
impl BatchFn<Uuid, Vec<User>> for UserBatcher {
  async fn load(&mut self, project_ids: &[Uuid]) -> HashMap<Uuid, Vec<User>> {
    let users = get_project_users(&self.pool, project_ids)
      .await
      .map(|db_users| -> HashMap<Uuid, Vec<User>> {
        project_ids
          .clone()
          .into_iter()
          .fold(HashMap::new(), |mut acc, project_id| {
            let users = db_users
              .clone()
              .into_iter()
              .filter(|db_user| {
                db_user
                  .project_ids
                  .contains(project_id)
              })
              .map(|db_user| db_user.into())
              .collect::<Vec<User>>();
            acc.insert(project_id.to_owned(), users);
            acc
          })
      })
      .unwrap();

    users
  }
}

pub type UserLoader = Loader<Uuid, Vec<User>, UserBatcher>;

// To create a new loader
pub fn get_user_loader(pool: PgPool) -> UserLoader {
  Loader::new(UserBatcher { pool }).with_yield_count(20)
}
