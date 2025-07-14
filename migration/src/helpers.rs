use crate::sea_orm::DatabaseBackend;
use sea_orm_migration::prelude::*;

pub fn is_postgres(manager: &SchemaManager) -> bool {
    let db = manager.get_connection();
    matches!(db.get_database_backend(), DatabaseBackend::Postgres)
}

pub fn is_sqllite(manager: &SchemaManager) -> bool {
    let db = manager.get_connection();
    matches!(db.get_database_backend(), DatabaseBackend::Sqlite)
}
