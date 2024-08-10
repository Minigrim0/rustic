use colog;
use log::info;

use crate::core::app::App;

// Initializes the music library
pub fn init() {
    colog::init();
}

pub fn default() -> App {
    init();
    info!("Loading default configuration");

    App::new()
}


pub fn from_file(path: &String) -> App {
    init();
    info!("Loading configuration from file: {}", path);

    App::new()
}
