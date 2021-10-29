// use crate::services::db::structure_service::get_structures;
// use async_trait::async_trait;
// use dataloader::non_cached::Loader;
// use dataloader::BatchFn;
// use sqlx::PgPool;
// use std::collections::HashMap;
// use uuid::Uuid;

// use crate::graphql::schema::structure::{Structure};

// pub struct StructureBatcher {
//   pool: PgPool,
// }

// #[async_trait]
// impl BatchFn<Uuid, Structure> for StructureBatcher {
//   async fn load(&mut self, ids: &[Uuid]) -> HashMap<Uuid, Structure> {
//     let structures = get_structures(&self.pool, ids)
//     .await
//     .map(|structures| -> HashMap<Uuid, Structure> {
//       structures
//         .into_iter()
//         .fold(HashMap::<Uuid, Structure>::new(), |mut acc, structure| {
//           let o: Structure = structure.into();
//           let k =  o.id.clone();
//           acc.insert(k, o);

//           acc
//         })
//     })
//     .unwrap();

//     structures
//   }
// }

// pub type StructureLoader = Loader<Uuid, Structure, StructureBatcher>;

// pub fn get_structure_loader(pool: PgPool) -> StructureLoader {
//   Loader::new(StructureBatcher { pool }).with_yield_count(20)
// }