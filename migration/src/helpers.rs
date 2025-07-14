use crate::sea_orm::DatabaseBackend;
use sea_orm_migration::prelude::*;

pub fn is_postgres(manager: &SchemaManager) -> bool {
    let db = manager.get_connection();
    if let DatabaseBackend::Postgres = db.get_database_backend() {
        true
    } else {
        false
    }
}

pub fn is_sqllite(manager: &SchemaManager) -> bool {
    let db = manager.get_connection();
    if let DatabaseBackend::Sqlite = db.get_database_backend() {
        true
    } else {
        false
    }
}
