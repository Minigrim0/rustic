use serde::Deserialize;
use std::default::Default;
use log::info;

use super::config::AppConfig;

#[derive(Deserialize, Default, Debug)]
/// Application meta-object, contains the application's configuration,
/// Available instruments, paths to save/load files to/from, ...
pub struct App {
    pub config: AppConfig,
}

impl App {
    pub fn new() -> App {
        App::default()
    }

    pub fn from_file(path: &String) -> App {
        info!("Loading configuration from file: {}", path);

        App::new()
    }

    pub fn run(&self) {
        info!("Running application");
    }
}
