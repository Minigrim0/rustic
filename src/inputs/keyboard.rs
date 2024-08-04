use evdev::{Device, Key};

pub fn find_keyboard() -> Option<Device> {
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
                    .map_or(false, |key| key.contains(Key::KEY_ENTER) && key.contains(Key::KEY_Q))
                {
                    println!("`{}` - OK", device.name().unwrap_or("Unknown device"));
                    break Some(device);
                } else {
                    println!("`{}` - NO", device.name().unwrap_or("Unknown device"));
                }
            }
            None => break None,
        }
    }
}
