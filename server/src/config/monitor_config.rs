use crate::config::MonitorGeneralConfig;
use crate::extensions::MappingExt;
use app::types::Monitor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorBase {
    #[serde(rename = "spec")]
    pub spec: serde_json::Value,
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    #[serde(rename = "kind")]
    pub kind: String,
    #[serde(rename = "sourceFile", default)]
    pub source_file: String,
    #[serde(rename = "name")]
    pub name: String,
    pub monitor_config: Option<MonitorGeneralConfig>,
}

impl MappingExt<Monitor> for MonitorBase {
    fn object_map(self) -> Monitor {
        Monitor {
            name: self.name,
            current_status: None,
            api_version: self.api_version,
            kind: self.kind,
            configuration: self.monitor_config.object_map(),
            spec: self.spec,
        }
    }
}
