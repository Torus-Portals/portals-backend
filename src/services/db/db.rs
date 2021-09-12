// use futures::Future;
// use sqlx::{PgPool, Postgres, Transaction};

// use anyhow::Result;

// // #[derive(Debug, Clone)]
// // pub struct DB<'db, Ex = PgPool>
// // where
// //   Ex: Executor<'db>,
// // {
// //   pub pool: Ex,
// // }

// // impl<'db, Ex> DB<'db, Ex> {
// //   pub fn new(pool: Ex) -> Self {
// //     DB { pool }
// //   }

// // }

// pub async fn transaction<'c, R, Fut: Future<Output = Result<R>>>(
//   pool: PgPool,
//   f: impl Fn(Transaction<'c, Postgres>) -> Fut,
// ) -> Result<R> {
//   // let tx: Transaction<'c, Postgres> = pool.begin().await?;
//   let tx = pool.begin().await?;

//   let r = f(tx).await;

//   // tx.commit().await?;

//   Ok(r)
// }
