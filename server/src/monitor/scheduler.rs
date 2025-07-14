use crate::monitor::tasks::{monitor_task, TaskFactory, TaskFactoryPtr};
use crate::signal::ExitSignaler;
use app::types::Monitor;
use app::DbFactoryPointer;
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::select;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

type MonitorTask = JoinHandle<Result<(), anyhow::Error>>;

#[derive(Debug)]
pub struct MonitorScheduler {
    monitor_tasks: Mutex<HashMap<String, MonitorTask>>,
    task_factory: TaskFactoryPtr,
    db_factory: DbFactoryPointer,
}

impl MonitorScheduler {
    pub fn new(db_factory: DbFactoryPointer) -> Self {
        Self {
            monitor_tasks: Mutex::new(HashMap::new()),
            task_factory: Arc::new(TaskFactory::new()),
            db_factory,
        }
    }

    pub async fn wait_for_shutdown(&self) -> Result<(), anyhow::Error> {
        let mut guard = self.monitor_tasks.lock().await;
        let tasks = guard.drain().map(|(_k, v)| v).collect::<Vec<_>>();
        select! {
            _ = join_all(tasks) => {},
            _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {}
        }

        Ok(())
    }

    pub async fn setup(&self) -> Result<(), anyhow::Error> {
        self.task_factory.register_standard_builders().await;

        Ok(())
    }

    pub async fn ensure_monitors_scheduled(
        &self,
        monitors: Vec<Monitor>,
        exit_signaler: ExitSignaler,
    ) -> Result<(), anyhow::Error> {
        let mut guard = self.monitor_tasks.lock().await;
        for monitor in monitors.into_iter() {
            if !guard.contains_key(&monitor.name) {
                let name = monitor.name.clone();
                let task = self.build_monitor_task(monitor, &exit_signaler);
                guard.insert(name, task);
            }
        }
        Ok(())
    }

    fn build_monitor_task(&self, monitor: Monitor, signaler: &ExitSignaler) -> MonitorTask {
        let monitor_handle = monitor_task(
            monitor,
            self.db_factory.clone(),
            self.task_factory.clone(),
            signaler.new_exit_signal(),
        );
        tokio::spawn(monitor_handle)
    }
}
