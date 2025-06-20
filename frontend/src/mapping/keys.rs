use sdl2::keyboard::{Keycode, Mod};
use serde::{Deserialize, Serialize};

/// Tracks the state of modifier keys with left/right distinction
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModifierState {
    pub left_ctrl: bool,
    pub right_ctrl: bool,
    pub left_shift: bool,
    pub right_shift: bool,
    pub left_alt: bool,
    pub right_alt: bool,
    pub left_gui: bool,
    pub right_gui: bool,
}

impl ModifierState {
    pub fn new() -> Self {
        Self {
            left_ctrl: false,
            right_ctrl: false,
            left_shift: false,
            right_shift: false,
            left_alt: false,
            right_alt: false,
            left_gui: false,
            right_gui: false,
        }
    }

    pub fn from_sdl_mod(sdl_mod: Mod) -> Self {
        Self {
            left_ctrl: sdl_mod.contains(Mod::LCTRLMOD),
            right_ctrl: sdl_mod.contains(Mod::RCTRLMOD),
            left_shift: sdl_mod.contains(Mod::LSHIFTMOD),
            right_shift: sdl_mod.contains(Mod::RSHIFTMOD),
            left_alt: sdl_mod.contains(Mod::LALTMOD),
            right_alt: sdl_mod.contains(Mod::RALTMOD),
            left_gui: sdl_mod.contains(Mod::LGUIMOD),
            right_gui: sdl_mod.contains(Mod::RGUIMOD),
        }
    }
}

/// Runtime key representation (optimized for performance)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyWithModifiers {
    pub keycode: Keycode,
    pub modifiers: ModifierState,
}

/// Serializable key representation using SDL2's name functions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SerializableKey {
    pub keycode_name: String,
    pub modifiers: ModifierState,
}

impl SerializableKey {
    pub fn from_runtime(key: &KeyWithModifiers) -> Self {
        Self {
            keycode_name: key.keycode.name(),
            modifiers: key.modifiers.clone(),
        }
    }

    pub fn to_runtime(&self) -> Option<KeyWithModifiers> {
        Keycode::from_name(&self.keycode_name).map(|keycode| KeyWithModifiers {
            keycode,
            modifiers: self.modifiers.clone(),
        })
    }
}
