use crate::extensions::*;
use app::types::{Monitor, MonitorConfiguration, MonitorStatus};
use entities::monitor::Model;
use entities::sea_orm_active_enums::Status;
use entities::{monitor, monitor_status};
use sea_orm::{NotSet, Set};

impl MappingExt<MonitorStatus> for monitor_status::Model {
    fn object_map(self) -> MonitorStatus {
        match self.status {
            Status::Up => MonitorStatus::Up {
                checked_at: self.created_at.into(),
            },
            Status::Down => MonitorStatus::Down {
                checked_at: self.created_at.into(),
                error_reason: self.error_reason.unwrap_or_default(),
            },
        }
    }
}

impl MappingExt<MonitorStatus> for Option<monitor_status::Model> {
    fn object_map(self) -> MonitorStatus {
        match self {
            None => MonitorStatus::Unknown,
            Some(st) => st.object_map(),
        }
    }
}

impl MappingExtraField1Ext<Monitor, Vec<monitor_status::Model>> for monitor::Model {
    fn object_map_field(self, extra: Vec<monitor_status::Model>) -> Monitor {
        let current_status = extra
            .into_iter()
            .max_by(|a, b| a.created_at.cmp(&b.created_at));

        let current_status = current_status.object_map();
        let current_status = Some(current_status);

        let api_version = self.api_version;
        let kind = self.kind;

        let configuration = self.check_interval.map(|interval| MonitorConfiguration {
            check_interval: Some(std::time::Duration::from_secs_f32(interval)),
        });

        Monitor {
            name: self.id,
            configuration,
            current_status,
            api_version,
            kind,
            spec: self.spec,
        }
    }
}

impl MappingExt<monitor::Model> for Monitor {
    fn object_map(self) -> Model {
        let check_interval = if let Some(cfg) = self.configuration {
            cfg.check_interval.map(|i| i.as_secs_f32())
        } else {
            None
        };

        Model {
            id: self.name,
            spec: self.spec,
            api_version: self.api_version,
            kind: self.kind,
            check_interval,
        }
    }
}

impl MappingExtraField1Ext<monitor_status::ActiveModel, String> for MonitorStatus {
    fn object_map_field(self, monitor_id: String) -> monitor_status::ActiveModel {
        let mut reason_val = NotSet;
        let monitor_status = match self {
            MonitorStatus::Up { .. } => Status::Up,
            MonitorStatus::Down { error_reason, .. } => {
                reason_val = Set(Some(error_reason));

                Status::Down
            }
            MonitorStatus::Unknown => {
                panic!("Unknown monitor status - we can't create this in the DB")
            }
        };

        monitor_status::ActiveModel {
            id: NotSet,
            created_at: NotSet,
            status: Set(monitor_status),
            monitor_id: Set(monitor_id),
            error_reason: reason_val,
        }
    }
}
