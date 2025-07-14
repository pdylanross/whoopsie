mod discovery;
mod scheduler;
mod tasks;

use crate::monitor::discovery::MonitorDiscovery;
use crate::monitor::scheduler::MonitorScheduler;
use crate::signal::ExitSignaler;
use app::state::ServerState;
use tokio::select;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct MonitorController {
    server_state: ServerState,
    discovery: MonitorDiscovery,
    scheduler: MonitorScheduler,
}

impl MonitorController {
    pub fn new(server_state: ServerState) -> Self {
        let discovery = MonitorDiscovery::new(&server_state);
        let scheduler = MonitorScheduler::new(server_state.db_factory.clone());
        Self {
            server_state,
            discovery,
            scheduler,
        }
    }

    pub fn start(self, exit_signaler: ExitSignaler) -> JoinHandle<Result<(), anyhow::Error>> {
        debug!("Starting MonitorController, cfg {:#?}", self);
        tokio::spawn(self.run(exit_signaler))
    }

    async fn run(self, exit_signaler: ExitSignaler) -> Result<(), anyhow::Error> {
        let mut error_count = 0;
        self.scheduler.setup().await?;

        loop {
            trace!("MonitorController is running");
            let mut signal = exit_signaler.new_exit_signal();

            if let Err(e) = self.run_iteration(exit_signaler.clone()).await {
                error!("MonitorController failed: {}", e);
                error_count += 1;
                if error_count > 20 {
                    return Err(e);
                }
            } else {
                error_count = 0;
            }

            select! {
                _ = signal.wait() => {
                    debug!("MonitorController received exit signal");
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
            }
        }

        self.scheduler.wait_for_shutdown().await?;
        Ok(())
    }

    async fn run_iteration(&self, exit_signaler: ExitSignaler) -> Result<(), anyhow::Error> {
        let mut monitors = self.discovery.discover().await?;

        for monitor in monitors.iter_mut() {
            if let Some(cfg) = &mut monitor.configuration {
                cfg.merge_with(&self.server_state.global_config)
            } else {
                monitor.configuration = Some(self.server_state.global_config.clone());
            }
        }

        self.scheduler
            .ensure_monitors_scheduled(monitors, exit_signaler)
            .await?;

        Ok(())
    }
}
