use evdev::{Device, Key};
use std::io;

fn find_keyboard() -> Option<Device> {
    let mut enumerator = evdev::enumerate();
    loop {
        let device_enum = enumerator.next();
        match device_enum {
            Some(denum) => {
                print!("Found device {:?} - ", &denum.0);
                let device = match Device::open(&denum.0) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("Unable to open device {:?} - {}", &denum.0, e);
                        continue;
                    }
                };
                if device
                    .supported_keys()
                    .map_or(false, |key| key.contains(Key::KEY_ENTER))
                {
                    println!("OK");
                    break Some(device);
                } else {
                    println!("NO");
                }
            }
            None => break None,
        }
    }
}

fn main() {
    match find_keyboard() {
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
}
