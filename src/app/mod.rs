use clap::Parser;

mod app;
mod cli;
mod config;
pub use app::App;

/// Initializes the application settings from the command line arguments.
/// This function is susceptible to terminate the process (e.g. when the command
/// line arguments ask for the application version or a dump of the config).
pub fn init_app() -> App {
    let args = cli::Cli::parse();
    let app = if let Some(path) = args.config {
        App::from_file(&path)
    } else {
        App::default()
    };

    if args.dump_config {
        match toml::to_string(&app.config) {
            Ok(s) => println!("{}", s),
            Err(e) => println!("Unable to dump config: {}", e.to_string()),
        }
        std::process::exit(0);
    }

    app
}
