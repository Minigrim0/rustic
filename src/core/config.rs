use serde::{Deserialize, Serialize};
use std::default::Default;

use crate::inputs::InputConfig;

#[derive(Debug, Deserialize, Serialize)]
/// Application Filesystem configuration
pub struct FSConfig {
    pub score_path: String,
    pub instrument_path: String,
    pub recordings_path: String,
}

impl Default for FSConfig {
    fn default() -> FSConfig {
        // TODO: Implement actual path building
        FSConfig {
            score_path: "{APP_ROOT_PATH}/scores/".to_string(),
            instrument_path: "{APP_ROOT_PATH}/instruments".to_string(),
            recordings_path: "{APP_ROOT_PATH}/recordings".to_string(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
/// The application configuration
pub struct AppConfig {
    pub input: InputConfig,
    pub fs: FSConfig,
    // pub sound_system: SoundSystemConfig,
}
