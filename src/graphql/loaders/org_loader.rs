use crate::services::db::org_service::DBOrg;
use crate::services::db::DB;
use async_trait::async_trait;
use dataloader::non_cached::Loader;
use dataloader::BatchFn;
use std::collections::HashMap;

use uuid::Uuid;

use crate::graphql::schema::org::Org;

pub struct OrgBatcher {
  db: DB,
}

#[async_trait]
impl BatchFn<Uuid, Org> for OrgBatcher {
  // async fn load(&mut self, ids: &[Uuid]) -> HashMap<Uuid, Org> {
  async fn load(&mut self, ids: &[Uuid]) -> HashMap<Uuid, Org> {
    // Question: How do I handle DB errors? HashMap<Uuid, Result<Org, Error>>?
    let orgs = self
      .db
      .get_orgs(ids)
      .await
      .map(|orgs| -> HashMap<Uuid, Org> {
        orgs
          .into_iter()
          .fold(HashMap::<Uuid, Org>::new(), |mut acc, org| {
            let o: Org = org.into();
            let k = o.id.clone();
            acc.insert(k, o);

            acc
          })
      })
      .unwrap();

    orgs
  }
}

pub type OrgLoader = Loader<Uuid, Org, OrgBatcher>;

// To create a new loader
pub fn get_org_loader(db: DB) -> OrgLoader {
  Loader::new(OrgBatcher { db }).with_yield_count(20)
}
