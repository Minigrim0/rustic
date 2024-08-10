use serde::Deserialize;
use std::default::Default;

use crate::core::config::Config;


#[derive(Deserialize, Default)]
/// Application meta-object, contains the application's configuration,
/// Available instruments, paths to save/load files to/from, ...
pub struct App {
    pub config: Config,
}


impl App {
    pub fn new() -> App {
        App::default()
    }
}
