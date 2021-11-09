use async_trait::async_trait;
use dataloader::non_cached::Loader;
use dataloader::BatchFn;
use itertools::Itertools;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::graphql::schema::dashboard::Dashboard;
use crate::services::db::dashboard_service::get_project_dashboards;

pub struct DashboardBatcher {
  pool: PgPool,
}

#[async_trait]
impl BatchFn<Uuid, Vec<Dashboard>> for DashboardBatcher {
  async fn load(&mut self, project_ids: &[Uuid]) -> HashMap<Uuid, Vec<Dashboard>> {
    let mut dashboards = get_project_dashboards(&self.pool, project_ids)
      .await
      .map(|db_dashboards| -> HashMap<Uuid, Vec<Dashboard>> {
        db_dashboards
          .into_iter()
          .map(|db_dashboard| (db_dashboard.project_id, Dashboard::from(db_dashboard)))
          .into_group_map()
      })
      .unwrap();

    // Add empty lists for projects that don't have dashboards
    project_ids
      .into_iter()
      .for_each(|project_id| {
        if !dashboards.contains_key(project_id) {
          let p_id = project_id.clone();
          dashboards.insert(p_id, vec![]);
        }
      });

    dashboards
  }
}

pub type DashboardLoader = Loader<Uuid, Vec<Dashboard>, DashboardBatcher>;

// To create a new loader
pub fn get_dashboard_loader(pool: PgPool) -> DashboardLoader {
  Loader::new(DashboardBatcher { pool }).with_yield_count(20)
}
