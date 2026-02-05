use crate::audio::{AudioBuffer, AudioLoader};
use crate::types::AudioSummary;

/// Application state: holds the currently loaded audio file and its precomputed summary.
/// Managed by Tauri via `RwLock<AudioState>`.
pub struct AudioState {
    pub buffer: Option<AudioBuffer>,
    pub summary: Option<AudioSummary>,
    pub loader: AudioLoader,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            buffer: None,
            summary: None,
            loader: AudioLoader::new(),
        }
    }
}

impl AudioState {
    /// Converts a time range (seconds) to a sample index range clamped to the buffer bounds.
    /// Returns `None` if no buffer is loaded or the range is empty.
    pub fn time_to_sample_range(&self, start: f64, end: f64) -> Option<(usize, usize)> {
        let buffer = self.buffer.as_ref()?;
        let sr = buffer.sample_rate() as f64;
        let total = buffer.samples().len();

        let s = (start * sr).round().max(0.0) as usize;
        let e = (end * sr).round().min(total as f64) as usize;

        if s >= e {
            return None;
        }

        Some((s, e))
    }
}