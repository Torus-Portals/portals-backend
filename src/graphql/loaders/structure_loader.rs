use crate::services::db::DB;
use async_trait::async_trait;
use dataloader::non_cached::Loader;
use dataloader::BatchFn;
use std::collections::HashMap;
use uuid::Uuid;

use crate::graphql::schema::structure::{Structure};

pub struct StructureBatcher {
  db: DB,
}

#[async_trait]
impl BatchFn<Uuid, Structure> for StructureBatcher {
  async fn load(&mut self, ids: &[Uuid]) -> HashMap<Uuid, Structure> {
    let structures = self
    .db
    .get_structures(ids)
    .await
    .map(|structures| -> HashMap<Uuid, Structure> {
      structures
        .into_iter()
        .fold(HashMap::<Uuid, Structure>::new(), |mut acc, structure| {
          let o: Structure = structure.into();
          let k = o.id.clone();
          acc.insert(k, o);

          acc
        })
    })
    .unwrap();

    structures
  }
}

pub type StructureLoader = Loader<Uuid, Structure, StructureBatcher>;

pub fn get_structure_loader(db: DB) -> StructureLoader {
  Loader::new(StructureBatcher { db }).with_yield_count(20)
}