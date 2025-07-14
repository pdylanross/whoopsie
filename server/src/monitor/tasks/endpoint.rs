use crate::monitor::tasks::{MonitorTask, TaskBuilder, TaskBuilderPtr, TaskPtr};
use anyhow::Error;
use app::types::{Monitor, MonitorStatus};
use migration::async_trait::async_trait;
use sea_orm::sqlx::types::chrono;
use sea_orm::sqlx::types::chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn get_builders() -> Vec<TaskBuilderPtr> {
    vec![Arc::new(V1Alpha1EndpointMonitorBuilder {})]
}

struct EndpointMonitor {
    pub uri: String,
}

#[async_trait]
impl MonitorTask for EndpointMonitor {
    async fn survey(&self) -> Result<MonitorStatus, Error> {
        let resp = reqwest::get(self.uri.as_str()).await;

        match resp {
            Ok(_) => Ok(MonitorStatus::Up {
                checked_at: chrono::Utc::now(),
            }),
            Err(err) => Ok(MonitorStatus::Down {
                checked_at: Utc::now(),
                error_reason: err.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct V1Alpha1EndpointMonitorSpec {
    pub uri: String,
}

#[derive(Debug)]
struct V1Alpha1EndpointMonitorBuilder {}

#[async_trait]
impl TaskBuilder for V1Alpha1EndpointMonitorBuilder {
    fn get_api_version(&self) -> String {
        "v1alpha1".to_string()
    }

    fn get_kind(&self) -> String {
        "endpoint".to_string()
    }

    async fn build(&self, monitor: Monitor) -> Result<TaskPtr, Error> {
        let spec = serde_json::from_value::<V1Alpha1EndpointMonitorSpec>(monitor.spec)?;

        Ok(Arc::new(EndpointMonitor { uri: spec.uri }))
    }
}
