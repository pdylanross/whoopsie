mod endpoint;

use crate::signal::ExitSignal;
use app::types::{Monitor, MonitorStatus};
use app::DbFactoryPointer;
use migration::async_trait::async_trait;
use sea_orm::sqlx::types::chrono::Utc;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::{RwLock, RwLockWriteGuard};

pub type TaskPtr = Arc<dyn MonitorTask + Send + Sync>;
pub type TaskBuilderPtr = Arc<dyn TaskBuilder + Send + Sync>;
pub type TaskFactoryPtr = Arc<TaskFactory>;

pub async fn monitor_task(
    monitor: Monitor,
    db: DbFactoryPointer,
    task_factory: TaskFactoryPtr,
    exit_signal: ExitSignal,
) -> Result<(), anyhow::Error> {
    let task = task_factory.construct_task(&monitor).await;

    if let Err(e) = task {
        log::error!("error constructing task: {e}");
        let monitor_repo = db.get_monitor_repository();
        let now = Utc::now();
        let status = MonitorStatus::Down {
            checked_at: now,
            error_reason: e.to_string(),
        };
        monitor_repo
            .log_status(monitor.name.clone(), status)
            .await?;
        return Err(e);
    }

    let task = task.unwrap();

    monitor_task_fn(monitor, task, db, exit_signal).await
}

async fn monitor_task_fn(
    monitor: Monitor,
    task: TaskPtr,
    db: DbFactoryPointer,
    mut exit_signal: ExitSignal,
) -> Result<(), anyhow::Error> {
    let loop_interval: Duration;
    if let Some(config) = monitor.configuration {
        if let Some(interval) = config.check_interval {
            loop_interval = interval;
        } else {
            panic!("check_interval not set");
        }
    } else {
        panic!("configuration not set");
    }

    loop {
        let monitor_repo = db.get_monitor_repository();
        let survey_result = task.survey().await;

        let mut status: MonitorStatus = MonitorStatus::Unknown;
        let now = Utc::now();
        if let Err(e) = survey_result {
            log::debug!("error surveying monitor {}: {}", monitor.name, e);
            status = MonitorStatus::Down {
                checked_at: now,
                error_reason: e.to_string(),
            };
        } else if let Ok(s) = survey_result {
            status = s
        }

        let log_result = monitor_repo.log_status(monitor.name.clone(), status).await;

        if let Err(e) = log_result {
            log::error!("error logging monitor status: {e}");
        }

        select! {
            _ = exit_signal.wait() => {
                return Ok(())
            }
            _ = tokio::time::sleep(loop_interval) => {}
        }
    }
}

#[async_trait]
pub trait MonitorTask {
    async fn survey(&self) -> Result<MonitorStatus, anyhow::Error>;
}

#[async_trait]
pub trait TaskBuilder: Debug {
    fn get_api_version(&self) -> String;
    fn get_kind(&self) -> String;
    async fn build(&self, monitor: Monitor) -> Result<TaskPtr, anyhow::Error>;
}

#[derive(Debug)]
pub struct TaskFactory {
    registrations: RwLock<HashMap<String, TaskBuilderPtr>>,
}

impl TaskFactory {
    pub fn new() -> Self {
        TaskFactory {
            registrations: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_standard_builders(&self) {
        let builders = endpoint::get_builders();
        self.bulk_register(builders).await;
    }

    #[allow(dead_code)]
    pub async fn list_registrations(&self) -> Vec<String> {
        let read_guard = self.registrations.read().await;
        read_guard.keys().cloned().collect()
    }

    pub async fn bulk_register(&self, builders: Vec<TaskBuilderPtr>) {
        let mut write_guard = self.registrations.write().await;
        for builder in builders {
            self.register_inner(&mut write_guard, builder).await;
        }
    }

    #[allow(dead_code)]
    pub async fn register(&self, builder: TaskBuilderPtr) {
        let mut write_guard = self.registrations.write().await;
        self.register_inner(&mut write_guard, builder).await;
    }

    async fn register_inner(
        &self,
        write_guard: &mut RwLockWriteGuard<'_, HashMap<String, TaskBuilderPtr>>,
        builder: TaskBuilderPtr,
    ) {
        let registration_name = self.builder_registration_name(&builder);
        write_guard.insert(registration_name, builder);
    }

    pub async fn construct_task(&self, monitor: &Monitor) -> Result<TaskPtr, anyhow::Error> {
        let registration_name = self.monitor_registration_name(monitor);
        let read_guard = self.registrations.read().await;

        if let Some(builder) = read_guard.get(&registration_name) {
            builder.build(monitor.clone()).await
        } else {
            Err(anyhow::anyhow!(
                "unknown monitor type: {}",
                registration_name
            ))
        }
    }

    fn monitor_registration_name(&self, monitor: &Monitor) -> String {
        format!("{}/{}", monitor.kind, monitor.api_version)
    }

    fn builder_registration_name(&self, builder: &TaskBuilderPtr) -> String {
        format!("{}/{}", builder.get_kind(), builder.get_api_version())
    }
}
