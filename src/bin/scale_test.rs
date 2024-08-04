use std::collections::HashMap;
use std::io::{self, Read};
use music::inputs;
use music::tones::{NOTES, TONES_FREQ};
use music::init;

fn main() -> io::Result<()>{
    init::init();

    let mut buf = [0u8; 1024];
    let mut stdin = io::stdin();

    let mappings: HashMap<u8, NOTES> = HashMap::from([
            (b'q', NOTES::C),
            (b'w', NOTES::CS),
            (b'e', NOTES::D),
            (b'r', NOTES::DS),
            (b't', NOTES::E),
            (b'y', NOTES::F),
            (b'u', NOTES::FS),
            (b'i', NOTES::G),
            (b'o', NOTES::GS),
            (b'p', NOTES::A),
            (b'[', NOTES::AS),
            (b']', NOTES::B),
        ]
    );

    let current_scale = 4;

    match inputs::keyboard::find_keyboard() {
        Some(mut keyboard) => loop {
            match keyboard.fetch_events() {
                Ok(events) => {
                    for event in events {
                        println!(
                            "Event:\n\t{:?}\t{:?}\t{:?}",
                            event.event_type(),
                            event.code(),
                            event.value()
                        );
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
