use crate::config::load_config;
use crate::config::monitor_config::MonitorBase;
use crate::extensions::MappingCloneExt;
use app::state::ServerState;
use app::types::{Monitor, RepositoryError};
use app::DbFactoryPointer;

#[derive(Debug)]
pub struct MonitorDiscovery {
    db_factory: DbFactoryPointer,
}

impl MonitorDiscovery {
    pub fn new(server_state: &ServerState) -> Self {
        let db_factory = server_state.db_factory.clone();
        Self { db_factory }
    }

    pub async fn discover(&self) -> Result<Vec<Monitor>, anyhow::Error> {
        self.discover_fs().await?;

        let monitors = self
            .db_factory
            .get_monitor_repository()
            .get_monitors()
            .await?;

        Ok(monitors)
    }

    async fn discover_fs(&self) -> Result<(), anyhow::Error> {
        // todo: this is super inefficient
        let config = load_config()?;

        if let Some(monitors) = config.monitors {
            for monitor in monitors {
                self.upsert_monitor_from_file_config(&monitor).await?;
            }
        }

        Ok(())
    }

    async fn upsert_monitor_from_file_config(
        &self,
        cfg: &MonitorBase,
    ) -> Result<(), anyhow::Error> {
        let new_monitor: Monitor = cfg.object_map_clone();
        let repo = self.db_factory.get_monitor_repository();
        let old_monitor = repo.get_monitor(new_monitor.name.clone()).await;
        match old_monitor {
            Ok(_) => {
                // todo: do a diff and update if needed
                Ok(())
            }
            Err(e) => match e {
                RepositoryError::NotFound(_) => {
                    repo.create_monitor(new_monitor).await?;
                    Ok(())
                }
                _ => Err(e.into()),
            },
        }
    }
}
