use crate::helpers::is_postgres;
use crate::sea_orm::{EnumIter, Iterable};
use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Monitor::Table)
                    .if_not_exists()
                    .col(string(Monitor::Id).primary_key().not_null())
                    .col(json(Monitor::Spec).not_null())
                    .col(string(Monitor::ApiVersion).not_null())
                    .col(string(Monitor::Kind).not_null())
                    .col(float_null(Monitor::CheckInterval).null())
                    .to_owned(),
            )
            .await?;

        if is_postgres(manager) {
            manager
                .create_type(
                    Type::create()
                        .as_enum(MonitorStatus::Status)
                        .values(MonitorStatusEnum::iter())
                        .to_owned(),
                )
                .await?;
        }

        manager
            .create_table(
                Table::create()
                    .table(MonitorStatus::Table)
                    .if_not_exists()
                    .col(pk_auto(MonitorStatus::Id))
                    .col(
                        timestamp_with_time_zone(MonitorStatus::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(MonitorStatus::Status)
                            .enumeration(
                                Alias::new(MonitorStatus::Status.to_string()),
                                MonitorStatusEnum::iter(),
                            )
                            .not_null(),
                    )
                    .col(string_null(MonitorStatus::ErrorReason))
                    .col(string(MonitorStatus::MonitorId).not_null())
                    .foreign_key(
                        &mut ForeignKey::create()
                            .from(MonitorStatus::Table, MonitorStatus::MonitorId)
                            .to(Monitor::Table, Monitor::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade)
                            .to_owned(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("monitor-status-monitor-id-create-date")
                    .table(MonitorStatus::Table)
                    .col(MonitorStatus::MonitorId)
                    .col(MonitorStatus::CreatedAt)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("monitor_status_monitor_id_fkey")
                    .table(MonitorStatus::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("monitor-status-monitor-id-create-date")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Monitor::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(MonitorStatus::Table).to_owned())
            .await?;

        if is_postgres(manager) {
            manager
                .drop_type(Type::drop().name(MonitorStatus::Status).to_owned())
                .await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Monitor {
    Table,
    Id,
    Spec,
    ApiVersion,
    Kind,
    CheckInterval,
}

#[derive(DeriveIden)]
enum MonitorStatus {
    Table,
    Id,
    MonitorId,
    Status,
    CreatedAt,
    ErrorReason,
}

#[derive(Iden, EnumIter)]
pub enum MonitorStatusEnum {
    #[iden = "up"]
    Up,
    #[iden = "down"]
    Down,
}
