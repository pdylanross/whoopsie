#[cfg(feature = "ssr")]
use crate::state::ServerState;
use leptos::prelude::*;
use std::time::Duration;

#[api_model]
pub struct Monitor {
    pub name: String,
    pub current_status: Option<MonitorStatus>,
    pub api_version: String,
    pub kind: String,
    pub configuration: Option<MonitorConfiguration>,
    pub spec: serde_json::Value,
}

#[api_model]
pub enum MonitorStatus {
    Up {
        checked_at: chrono::DateTime<chrono::Utc>,
    },
    Down {
        checked_at: chrono::DateTime<chrono::Utc>,
        error_reason: String,
    },
    #[default]
    Unknown,
}

#[server]
pub async fn get_monitors() -> Result<Vec<Monitor>, ServerFnError> {
    let server_state = expect_context::<ServerState>();
    let monitor_repository = server_state.db_factory.get_monitor_repository();

    Ok(monitor_repository.get_monitors().await?)
}

#[server]
pub async fn get_monitor(id: String) -> Result<Monitor, ServerFnError> {
    let server_state = expect_context::<ServerState>();
    let monitor_repository = server_state.db_factory.get_monitor_repository();

    Ok(monitor_repository.get_monitor(id).await?)
}

#[api_model]
pub struct MonitorConfiguration {
    #[serde(with = "humantime_serde")]
    pub check_interval: Option<Duration>,
}

impl MonitorConfiguration {
    pub fn merge_with(&mut self, other: &MonitorConfiguration) {
        if let Some(interval) = other.check_interval {
            if self.check_interval.is_none() {
                self.check_interval = Some(interval);
            }
        }
    }
}
