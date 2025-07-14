use crate::db::seaorm_repositories::errors::RepoErrExts;
use crate::extensions::*;
use app::types::{Monitor, MonitorRepository, MonitorStatus, RepositoryError};
use entities::{monitor, monitor_status};
use migration::async_trait::async_trait;
use sea_orm::prelude::*;
use sea_orm::QueryOrder;
use std::sync::Arc;

pub fn new_monitor_repository(db: Arc<DbConn>) -> Arc<impl MonitorRepository + Send + Sync> {
    Arc::new(SeaormMonitorRepository { db })
}

struct SeaormMonitorRepository {
    db: Arc<DbConn>,
}

#[async_trait]
impl MonitorRepository for SeaormMonitorRepository {
    async fn get_monitors(&self) -> Result<Vec<Monitor>, RepositoryError> {
        let results = monitor::Entity::find()
            .find_with_related(monitor_status::Entity)
            .order_by_desc(monitor_status::Column::CreatedAt)
            .all(self.db.as_ref())
            .await
            .to_repo_err()?;

        let ret = results
            .into_iter()
            .map(|(m, mst)| m.object_map_field(mst))
            .collect();

        Ok(ret)
    }

    async fn get_monitor(&self, id: String) -> Result<Monitor, RepositoryError> {
        let results = monitor::Entity::find_by_id(id.clone())
            .find_with_related(monitor_status::Entity)
            .order_by_desc(monitor_status::Column::CreatedAt)
            .all(self.db.as_ref())
            .await
            .to_repo_err()?;

        let ret = results
            .into_iter()
            .map(|(m, mst)| m.object_map_field(mst))
            .next();

        match ret {
            None => Err(RepositoryError::NotFound(id)),
            Some(res) => Ok(res),
        }
    }

    async fn create_monitor(&self, monitor: Monitor) -> Result<(), RepositoryError> {
        let model: monitor::ActiveModel = monitor.object_map().into();

        let _ = model.insert(self.db.as_ref()).await.to_repo_err()?;

        Ok(())
    }

    async fn log_status(
        &self,
        monitor_id: String,
        status: MonitorStatus,
    ) -> Result<(), RepositoryError> {
        let new_status = status.object_map_field(monitor_id);
        new_status.insert(self.db.as_ref()).await.to_repo_err()?;
        Ok(())
    }
}
