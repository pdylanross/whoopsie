use crate::config::database_config::DatabaseConfigBase;
use crate::db::seaorm_repositories::monitor::new_monitor_repository;
use app::types::{DbFactory, MonitorRepository};
use app::DbFactoryPointer;
use migration::async_trait::async_trait;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde::Deserialize;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

mod errors;
mod mappings;
mod monitor;

struct SeaOrmDbFactory {
    db: Arc<DatabaseConnection>,

    monitor_repository: Arc<dyn MonitorRepository + Send + Sync>,
}

impl Debug for SeaOrmDbFactory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SeaOrmDbFactory")
            // We can debug format the Arc<DatabaseConnection>
            .field("db", &self.db)
            .finish()
    }
}

#[async_trait]
impl DbFactory for SeaOrmDbFactory {
    async fn initialize_db(&self) -> Result<(), anyhow::Error> {
        Migrator::up(self.db.as_ref(), None).await?;
        Ok(())
    }

    fn get_monitor_repository(&self) -> Arc<dyn MonitorRepository + Send + Sync> {
        self.monitor_repository.clone()
    }
}

pub async fn build(
    connect_options: Arc<dyn SeaOrmConnectOptions>,
) -> Result<DbFactoryPointer, anyhow::Error> {
    let mut opt = ConnectOptions::new(connect_options.get_connect_url());
    connect_options.configure(&mut opt);

    debug!("Connecting to database: {:#?}", opt);

    let db = Arc::new(Database::connect(opt).await?);

    let monitor_repository = new_monitor_repository(db.clone());

    Ok(Arc::new(SeaOrmDbFactory {
        db,
        monitor_repository,
    }))
}

pub trait SeaOrmConnectOptions: Debug {
    fn configure(&self, options: &mut ConnectOptions);
    fn get_connect_url(&self) -> String;
}

pub fn try_load_config(
    config: &Option<DatabaseConfigBase>,
) -> Result<Option<Arc<dyn SeaOrmConnectOptions>>, anyhow::Error> {
    if config.is_none() {
        return Ok(Some(SqliteConnectOptions::new_in_memory()));
    }

    let config = config.as_ref().unwrap();
    if config.api_version == "v1" {
        if config.kind == "sqlite" {
            return Ok(Some(SqliteConnectOptions::from_config_base(config)?));
        }

        if config.kind == "postgresql" {
            return Ok(Some(PostgresqlConnectOptions::from_config_base(config)?));
        }
    }

    Ok(None)
}

#[derive(Debug, Clone, Deserialize)]
pub struct SqliteConnectOptions {
    pub in_memory: Option<bool>,
    pub file_path: Option<String>,
}

impl SeaOrmConnectOptions for SqliteConnectOptions {
    fn configure(&self, _: &mut ConnectOptions) {}

    fn get_connect_url(&self) -> String {
        if let Some(in_memory) = self.in_memory {
            if in_memory {
                return "sqlite::memory:".to_string();
            }
        }

        if let Some(file_path) = self.file_path.as_ref() {
            return format!("sqlite:/{file_path}?mode=rwc");
        }

        panic!("Invalid SqliteConnectOptions, should be validated pre this code path");
    }
}

impl SqliteConnectOptions {
    pub fn new_in_memory() -> Arc<Self> {
        Arc::new(Self {
            in_memory: Some(true),
            file_path: None,
        })
    }

    pub fn from_config_base(config: &DatabaseConfigBase) -> Result<Arc<Self>, anyhow::Error> {
        let res = Self::deserialize(&config.spec)?;

        if res.in_memory.is_none() && res.file_path.is_none() {
            return Err(anyhow::anyhow!(
                "Invalid SqliteConnectOptions - either in_memory or file_path must be set"
            ));
        }

        Ok(Arc::new(res))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresqlConnectOptions {}

impl SeaOrmConnectOptions for PostgresqlConnectOptions {
    fn configure(&self, _options: &mut ConnectOptions) {
        todo!()
    }

    fn get_connect_url(&self) -> String {
        todo!()
    }
}

impl PostgresqlConnectOptions {
    pub fn from_config_base(_config: &DatabaseConfigBase) -> Result<Arc<Self>, anyhow::Error> {
        todo!()
    }
}
