use log::info;
use serde::Deserialize;
use std::default::Default;

pub mod cli;
pub mod config;
pub mod keys;
pub mod macros;
pub mod note;
pub mod score;
pub mod tones;

use config::AppConfig;

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

    pub fn default() -> App {
        info!("Loading default configuration");

        App::new()
    }

    pub fn from_file(path: &String) -> App {
        info!("Loading configuration from file: {}", path);

        App::new()
    }

    pub fn run(&self) {
        info!("Running application");
    }
}
