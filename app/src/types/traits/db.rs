use crate::types::{Monitor, MonitorStatus};
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;

#[async_trait]
pub trait DbFactory: Debug {
    async fn initialize_db(&self) -> Result<(), anyhow::Error>;
    fn get_monitor_repository(&self) -> Arc<dyn MonitorRepository + Send + Sync>;
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error("Object not found: {0}")]
    NotFound(String),
}

#[async_trait]
pub trait MonitorRepository {
    async fn get_monitors(&self) -> Result<Vec<Monitor>, RepositoryError>;
    async fn get_monitor(&self, id: String) -> Result<Monitor, RepositoryError>;
    async fn create_monitor(&self, monitor: Monitor) -> Result<(), RepositoryError>;

    async fn log_status(
        &self,
        monitor_id: String,
        status: MonitorStatus,
    ) -> Result<(), RepositoryError>;
}
