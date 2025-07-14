use crate::config::AppConfig;
use crate::types::MonitorConfiguration;
use crate::DbFactoryPointer;
use axum::extract::FromRef;
use leptos::config::LeptosOptions;

#[derive(FromRef, Debug, Clone)]
pub struct ServerState {
    pub leptos_options: LeptosOptions,
    pub app_config: AppConfig,
    pub db_factory: DbFactoryPointer,
    pub global_config: MonitorConfiguration,
}
