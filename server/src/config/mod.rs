pub mod database_config;
pub mod monitor_config;

use crate::config::database_config::DatabaseConfigBase;
use crate::config::monitor_config::MonitorBase;
use crate::extensions::MappingExt;
use app::config::AppConfig;
use app::types::MonitorConfiguration;
use figment::providers::Format;
use figment::{
    providers::{Env, YamlExtended},
    Error as FigmentError, Figment,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::Path;
use std::time::Duration;
use std::{env, fs};

const CONFIG_DIR_ENV_VAR: &str = "CONFIG_DIR";
const ENV_PREFIX: &str = "WHOOPS";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerConfig {
    pub app_config: Option<AppConfig>,
    pub monitors: Option<Vec<MonitorBase>>,
    pub db: Option<DatabaseConfigBase>,
    pub global_monitor_config: Option<MonitorGeneralConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorGeneralConfig {
    #[serde(with = "humantime_serde")]
    pub check_interval: Option<Duration>,
}

impl Default for MonitorGeneralConfig {
    fn default() -> Self {
        MonitorGeneralConfig {
            check_interval: Some(Duration::from_secs(30)),
        }
    }
}

impl MappingExt<MonitorConfiguration> for MonitorGeneralConfig {
    fn object_map(self) -> MonitorConfiguration {
        MonitorConfiguration {
            check_interval: self.check_interval,
        }
    }
}

impl MappingExt<Option<MonitorConfiguration>> for Option<MonitorGeneralConfig> {
    fn object_map(self) -> Option<MonitorConfiguration> {
        self.map(|config| config.object_map())
    }
}

impl ServerConfig {
    pub fn set_source_file(&mut self, path: Cow<str>) {
        if let Some(monitors) = self.monitors.as_mut() {
            for m in monitors {
                m.source_file = path.to_string();
            }
        }
    }

    pub fn merge(&mut self, other: ServerConfig) {
        // merge monitor config arrays
        if let Some(new_monitors) = other.monitors {
            if let Some(old_monitors) = self.monitors.as_mut() {
                old_monitors.extend(new_monitors);
            } else {
                self.monitors = Some(new_monitors.clone());
            }
        }

        // app config takes the last loaded config value
        if let Some(new_app_config) = other.app_config {
            self.app_config = Some(new_app_config);
        }

        // DB Config takes the last loaded value
        if let Some(new_db_config) = other.db {
            self.db = Some(new_db_config);
        }
    }
}

#[allow(clippy::result_large_err)]
pub fn load_config() -> Result<ServerConfig, ConfigError> {
    let config = load_config_from_dir(CONFIG_DIR_ENV_VAR, Some(ENV_PREFIX))?;

    debug!("Config: {:?}", config);
    Ok(config)
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Environment variable '{var_name}' is not set")]
    EnvVarNotSet { var_name: String },

    #[error("Config directory '{path}' does not exist")]
    DirectoryNotExists { path: String },

    #[error("Config path '{path}' is not a directory")]
    NotADirectory { path: String },

    #[error("Failed to read config directory '{path}': {source}")]
    DirectoryReadError {
        path: String,
        source: std::io::Error,
    },

    #[error("Failed to read directory entry: {source}")]
    DirectoryEntryError { source: std::io::Error },

    #[error("No YAML files found in config directory '{path}'")]
    NoYamlFiles { path: String },

    #[error(transparent)]
    FigmentError(#[from] FigmentError),
}

#[allow(clippy::result_large_err)]
fn load_config_from_dir(
    config_dir_env_var: &str,
    env_prefix: Option<&str>,
) -> Result<ServerConfig, ConfigError> {
    // Get the config directory from environment variable
    let config_dir = env::var(config_dir_env_var).map_err(|_| ConfigError::EnvVarNotSet {
        var_name: config_dir_env_var.to_string(),
    })?;

    let config_path = Path::new(&config_dir);

    // Check if directory exists
    if !config_path.exists() {
        return Err(ConfigError::DirectoryNotExists {
            path: config_dir.clone(),
        });
    }

    if !config_path.is_dir() {
        return Err(ConfigError::NotADirectory {
            path: config_dir.clone(),
        });
    }

    // Read directory and collect YAML files
    let mut yaml_files = Vec::new();

    let entries = fs::read_dir(config_path).map_err(|e| ConfigError::DirectoryReadError {
        path: config_dir.clone(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| ConfigError::DirectoryEntryError { source: e })?;

        let path = entry.path();

        // Only include files (not directories) with yaml/yml extensions
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "yaml" || extension == "yml" {
                    yaml_files.push(path);
                }
            }
        }
    }

    // Check if we found any YAML files
    if yaml_files.is_empty() {
        return Err(ConfigError::NoYamlFiles {
            path: config_dir.clone(),
        });
    }

    // Sort files alphanumerically for consistent loading order
    yaml_files.sort();

    // Start building the config
    let mut ret = ServerConfig::default();

    // Add each YAML file to the figment in order
    for yaml_file in yaml_files {
        debug!("Loading config from: {:?}", yaml_file);
        let mut new_config: ServerConfig = Figment::new()
            .merge(YamlExtended::file(&yaml_file))
            .extract()?;

        new_config.set_source_file(yaml_file.to_string_lossy());

        ret.merge(new_config);
    }

    // Add environment variable provider last (highest precedence)
    let env_config = if let Some(prefix) = env_prefix {
        debug!("Using environment variable prefix: {}", prefix);
        Figment::new().merge(Env::prefixed(prefix)).extract()?
    } else {
        Figment::new().merge(Env::raw()).extract()?
    };
    ret.merge(env_config);

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_config_from_dir_success() {
        // Create a temporary directory with test YAML files
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path();

        // Create test YAML files
        fs::write(
            config_dir.join("01-database.yaml"),
            "database:\n  url: postgres://localhost\n  pool_size: 10",
        )
        .unwrap();

        fs::write(
            config_dir.join("02-monitoring.yaml"),
            "monitoring:\n  interval: 30\n  max_monitors: 1000",
        )
        .unwrap();

        fs::write(
            config_dir.join("99-overrides.yaml"),
            "database:\n  pool_size: 20", // This should override the pool_size from 01-database.yaml
        )
        .unwrap();

        // Set environment variable
        env::set_var("TEST_CONFIG_DIR", config_dir.to_str().unwrap());

        // Load configuration
        let _figment = load_config_from_dir("TEST_CONFIG_DIR", Some("TEST")).unwrap();

        // The figment should be ready to extract configuration
        // In a real application, you would extract into your config struct here

        // Clean up
        env::remove_var("TEST_CONFIG_DIR");
    }

    #[test]
    fn test_load_config_env_var_not_set() {
        // Ensure the environment variable is not set
        env::remove_var("NONEXISTENT_CONFIG_DIR");

        let result = load_config_from_dir("NONEXISTENT_CONFIG_DIR", None);
        assert!(result.is_err());

        let error = result.unwrap_err();

        match error {
            ConfigError::EnvVarNotSet { var_name } => {
                assert_eq!(var_name, "NONEXISTENT_CONFIG_DIR");
            }
            _ => panic!("Expected EnvVarNotSet error"),
        }
    }

    #[test]
    fn test_load_config_directory_not_exists() {
        // Set environment variable to non-existent directory
        env::set_var("TEST_CONFIG_DIR_BAD", "/nonexistent/directory");

        let result = load_config_from_dir("TEST_CONFIG_DIR_BAD", None);
        assert!(result.is_err());

        let error = result.unwrap_err();

        match error {
            ConfigError::DirectoryNotExists { path } => {
                assert_eq!(path, "/nonexistent/directory");
            }
            _ => panic!("Expected DirectoryNotExists error"),
        }

        // Clean up
        env::remove_var("TEST_CONFIG_DIR_BAD");
    }

    #[test]
    fn test_load_config_no_yaml_files() {
        // Create a temporary directory with no YAML files
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path();

        // Create a non-YAML file
        fs::write(config_dir.join("config.txt"), "not a yaml file").unwrap();

        // Set environment variable
        env::set_var("TEST_CONFIG_DIR_EMPTY", config_dir.to_str().unwrap());

        let result = load_config_from_dir("TEST_CONFIG_DIR_EMPTY", None);
        assert!(result.is_err());

        let error = result.unwrap_err();

        match error {
            ConfigError::NoYamlFiles { path } => {
                assert_eq!(path, config_dir.to_str().unwrap());
            }
            _ => panic!("Expected NoYamlFiles error"),
        }

        // Clean up
        env::remove_var("TEST_CONFIG_DIR_EMPTY");
    }
}
