use crate::error::AppError;
use rustic::app::prelude::FSConfig;
use rustic::app::prelude::SystemConfig;
use rustic::audio::{AudioConfig, LogConfig};
use serde::{Deserialize, Serialize};

/// Mirrors `AppConfig` from the engine but only the user-facing fields.
/// Kept in sync manually; the engine's `AppConfig` includes internal fields (fs paths)
/// that the frontend should not touch.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineConfig {
    #[serde(default)]
    pub system: SystemConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub logging: LogConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig::default(),
            audio: AudioConfig::default(),
            logging: LogConfig::default(),
        }
    }
}

fn config_file_path() -> Result<std::path::PathBuf, AppError> {
    let root = FSConfig::app_root_dir().map_err(|e| AppError::ConfigError(e.to_string()))?;
    Ok(root.join("config.toml"))
}

#[tauri::command]
pub fn get_engine_config() -> Result<EngineConfig, AppError> {
    let path = config_file_path()?;
    if !path.exists() {
        return Ok(EngineConfig::default());
    }
    let contents =
        std::fs::read_to_string(&path).map_err(|e| AppError::ConfigError(e.to_string()))?;
    let config: EngineConfig =
        toml::from_str(&contents).map_err(|e| AppError::ConfigError(e.to_string()))?;
    Ok(config)
}

#[tauri::command]
pub fn set_engine_config(config: EngineConfig) -> Result<(), AppError> {
    let path = config_file_path()?;
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let toml_string =
        toml::to_string_pretty(&config).map_err(|e| AppError::ConfigError(e.to_string()))?;
    std::fs::write(&path, toml_string)?;
    log::info!("Engine config saved to {}", path.display());
    Ok(())
}
