use clap::Parser;
use evdev::EventType;
use log::info;
use std::io;

use rustic::core::cli::Cli;
use rustic::core::App;
use rustic::inputs;

fn main() -> io::Result<()> {
    let args = Cli::parse();
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
        return Ok(());
    }

    let mapping = inputs::keyboard::get_mapping();

    match inputs::keyboard::find_keyboard() {
        Some(mut keyboard) => loop {
            match keyboard.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if event.event_type() == EventType::KEY && event.value() != 2 {
                            if mapping.contains_key(&event.code()) {
                                println!("{:?}", mapping[&event.code()]);
                            } else {
                                println!(
                                    "Event:\n\t{:?}\t{:?}\t{:?}",
                                    event.event_type(),
                                    event.code(),
                                    event.value()
                                );
                            }
                        }
                    }
                }
                Err(_) => {
                    info!("Error fetching events");
                }
            }
        },
        None => {
            println!("No keyboard could be found");
        }
    }

    Ok(())
}
