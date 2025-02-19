mod drum;

pub mod prelude {
    pub use super::drum::*;
}

pub trait Instrument {
    /// Starts playing the given note
    fn start_note(&self, note: u8);

    /// Stops playing the given note
    fn stop_note(&self, note: u8);

    /// Returns the current output of the instrument
    fn get_output(&self) -> f32;
}
