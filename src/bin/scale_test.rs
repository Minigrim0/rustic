use evdev::EventType;
use music::init;
use music::inputs;
use music::tones::NOTES;
use std::io::{self};

fn main() -> io::Result<()> {
    init::init();

    let buf = [0u8; 1024];
    let stdin = io::stdin();

    let mapping = inputs::keyboard::get_mapping();

    let current_scale = 4;

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
                Err(_) => {}
            }
        },
        None => {
            println!("No keyboard could be found");
        }
    }

    Ok(())
}
