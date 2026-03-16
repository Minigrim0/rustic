//! Keyboard input handling for Linux systems.

#[cfg(feature = "input")]
use evdev::{Device, EventType, Key};
#[cfg(feature = "input")]
use std::path::Path;

/// A discovered input device, which may or may not be accessible.
#[cfg(feature = "input")]
pub enum InputDevice {
    /// Device was opened successfully.
    Available { label: String, device: Device },
    /// Device exists but could not be opened (e.g. permission denied).
    Inaccessible { label: String },
}

#[cfg(feature = "input")]
impl InputDevice {
    pub fn label(&self) -> &str {
        match self {
            InputDevice::Available { label, .. } => label,
            InputDevice::Inaccessible { label } => label,
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self, InputDevice::Available { .. })
    }

    /// Consume the entry and return the inner `Device`, if available.
    pub fn into_device(self) -> Option<Device> {
        match self {
            InputDevice::Available { device, .. } => Some(device),
            InputDevice::Inaccessible { .. } => None,
        }
    }
}

/// List every `/dev/input/event*` node, whether or not it can be opened.
///
/// Inaccessible entries are included so the caller can tell the user about
/// permission issues. Entries are sorted by path.
///
/// Only available with the `input` feature enabled.
#[cfg(feature = "input")]
pub fn list_input_devices() -> Vec<InputDevice> {
    let devices_dir = Path::new("/dev/input");
    let mut result = Vec::new();

    let Ok(read_dir) = std::fs::read_dir(devices_dir) else {
        return result;
    };

    let mut entries: Vec<_> = read_dir.flatten().collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !file_name.starts_with("event") {
            continue;
        }

        match Device::open(&path) {
            Ok(device) => {
                let name = device.name().unwrap_or("unknown").to_string();
                let label = format!("{file_name}: {name}");
                result.push(InputDevice::Available { label, device });
            }
            Err(_) => {
                let label = format!("{file_name}: (not accessible — check permissions)");
                result.push(InputDevice::Inaccessible { label });
            }
        }
    }

    result
}

/// Find the first keyboard-like device (supports `KEY_Q` and `KEY_W`).
///
/// Only available with the `input` feature enabled.
#[cfg(feature = "input")]
pub fn find_keyboard() -> Option<Device> {
    let devices_dir = Path::new("/dev/input");

    let mut entries: Vec<_> = std::fs::read_dir(devices_dir).ok()?.flatten().collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str())?;
        if !file_name.starts_with("event") {
            continue;
        }
        let Ok(device) = Device::open(&path) else {
            continue;
        };
        if !device.supported_events().contains(EventType::KEY) {
            continue;
        }
        if let Some(keys) = device.supported_keys() {
            if keys.contains(Key::KEY_Q) && keys.contains(Key::KEY_W) {
                return Some(device);
            }
        }
    }

    None
}
