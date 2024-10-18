pub mod core;
pub mod tones;
pub mod score;

pub mod generator;

// Should be last, maps inputs to functions of the previous mods
pub mod inputs;

#[cfg(feature = "plotting")]
pub mod plotting;
