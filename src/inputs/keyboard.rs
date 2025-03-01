use evdev::{Device, Key};
use std::collections::HashMap;

use log::{info, warn};

use crate::core::keys;
use crate::note;

pub fn get_mapping() -> HashMap<u16, keys::Key> {
    HashMap::from([
        (16, note!(keys::KeyCode::NoteC)),
        (17, note!(keys::KeyCode::NoteCS)),
        (18, note!(keys::KeyCode::NoteD)),
        (19, note!(keys::KeyCode::NoteDS)),
        (20, note!(keys::KeyCode::NoteE)),
        (21, note!(keys::KeyCode::NoteF)),
        (22, note!(keys::KeyCode::NoteFS)),
        (23, note!(keys::KeyCode::NoteG)),
        (24, note!(keys::KeyCode::NoteGS)),
        (25, note!(keys::KeyCode::NoteA)),
        (26, note!(keys::KeyCode::NoteAS)),
        (27, note!(keys::KeyCode::NoteB)),
    ])
}

pub fn list_devices() {
    let enumerator = evdev::enumerate();
    for d in enumerator {
        match Device::open(&d.0) {
            Ok(device) => {
                info!("Device {:?} - `{}`", &d.0, device.name().unwrap_or("Unknown device"));
            },
            Err(e) => {
                warn!("Unable to open device {:?} - {}", &d.0, e);
            }
        };
    }
}

pub fn find_keyboard() -> Option<Device> {
    list_devices();
    let mut enumerator = evdev::enumerate();
    loop {
        let device_enum = enumerator.next();
        match device_enum {
            Some(denum) => {
                info!("Found device {:?} - ", &denum.0);
                let device = match Device::open(&denum.0) {
                    Ok(d) => d,
                    Err(e) => {
                        warn!("Unable to open device {:?} - {}", &denum.0, e);
                        continue;
                    }
                };
                if !device.name().unwrap_or("").contains("virtual") && device.supported_keys().map_or(false, |key| {
                    key.contains(Key::KEY_ENTER) && key.contains(Key::KEY_Q)
                }) {
                    info!("`{}` - OK", device.name().unwrap_or("Unknown device"));
                    break Some(device);
                } else {
                    info!("`{}` - NO", device.name().unwrap_or("Unknown device"));
                }
            }
            None => break None,
        }
    }
}
