use chrono::{DateTime, NaiveDateTime, Utc};

use super::mutation::Mutation;
use super::query::Query;

use crate::models::org::Org;
use crate::models::user::User;
use crate::{graphql::context::GQLContext, models::org::NewOrg};
use sqlx;
use uuid::Uuid;

impl Query {
  pub async fn orgs_impl(ctx: &GQLContext) -> Vec<Org> {
    let rows = sqlx::query_as!(Org, "select * from orgs")
      .fetch_all(&ctx.pool)
      .await;

    match rows {
      Ok(r) => r,
      Err(e) => {
        vec![]
      }
    }
  }

  pub async fn org_impl(ctx: &GQLContext, id: Uuid) -> Org {
    let row = sqlx::query_as!(Org, "select * from orgs where id = $1", id)
      .fetch_one(&ctx.pool)
      .await;

    match row {
      Ok(r) => r,
      Err(e) => {
        println!("error parsing row: {:?}", e);
        // Need to get actual errors working.
        Org {
          ..Default::default()
        }
      }
    }
  }
}

impl Mutation {
  pub async fn create_org_impl(ctx: &GQLContext, new_org: NewOrg) -> Org {
    let row = sqlx::query_as!(
      Org,
      r#"
      with _user as (select * from users where auth0id = $1)
      insert into orgs (name, created_by, updated_by) values ($2, (select id from _user), (select id from _user))
      returning name, id, created_at, created_by, updated_at, updated_by
      "#,
      &ctx.auth0_user_id,
      new_org.name
    )
    .fetch_one(&ctx.pool)
    .await;

    match row {
      Ok(r) => r,
      Err(e) => {
        println!("error parsing row: {:?}", e);
        // Need to get actual errors working.
        Org {
          ..Default::default()
        }
      }
    }
  }
}
