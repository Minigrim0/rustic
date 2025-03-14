use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Note {
    Silence,   // 1 time silence
    HalfPause, // 2 times
    Pause,     // 4 times
}
