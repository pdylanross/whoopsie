mod seaorm_repositories;

use crate::config::database_config::DatabaseConfigBase;
use app::DbFactoryPointer;

pub async fn get_db_factory(
    config: &Option<DatabaseConfigBase>,
) -> Result<DbFactoryPointer, anyhow::Error> {
    if let Some(opt) = seaorm_repositories::try_load_config(config)? {
        debug!("Database config loaded {:#?}", opt);
        seaorm_repositories::build(opt).await
    } else {
        Err(anyhow::anyhow!(
            "Malformed database config, could not connect to database"
        ))
    }
}
