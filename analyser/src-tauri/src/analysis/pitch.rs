use log::info;

/// Estimate pitch using autocorrelation
pub fn estimate_pitch(samples: &[f32], sample_rate: u32) -> Option<f32> {
    if samples.is_empty() {
        return None;
    }

    // Use a section of the sample for pitch detection
    let window_size = sample_rate as usize / 10; // 100ms window
    let samples_to_use = if samples.len() > window_size {
        &samples[0..window_size]
    } else {
        samples
    };

    // Compute autocorrelation
    let mut autocorrelation = Vec::with_capacity(samples_to_use.len());
    for lag in 0..samples_to_use.len() {
        let mut sum = 0.0;
        for i in 0..(samples_to_use.len() - lag) {
            sum += samples_to_use[i] * samples_to_use[i + lag];
        }
        autocorrelation.push(sum);
    }

    // Find the highest peak after the first zero crossing
    let mut zero_crossed = false;
    let mut max_val = 0.0;
    let mut max_lag = 0;

    for (lag, &val) in autocorrelation.iter().enumerate().skip(1) {
        if !zero_crossed && val <= 0.0 {
            zero_crossed = true;
        } else if zero_crossed && val > max_val {
            max_val = val;
            max_lag = lag;
        }
    }

    // Calculate pitch
    if max_lag > 0 {
        let pitch = sample_rate as f32 / max_lag as f32;
        info!("Estimated pitch: {:.2} Hz", pitch);
        Some(pitch)
    } else {
        None
    }
}

/// Converts frequency to musical note
pub fn frequency_to_note(frequency: f32) -> String {
    // A4 = 440Hz is our reference
    const A4_FREQ: f32 = 440.0;
    const A4_MIDI: i32 = 69; // MIDI note number for A4

    // Note names with sharps
    const NOTE_NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    if frequency <= 0.0 {
        return "?".to_string();
    }

    // Calculate MIDI note number (which can be fractional)
    let midi_note = 12.0 * (frequency / A4_FREQ).log2() + A4_MIDI as f32;

    // Get the closest MIDI note
    let closest_midi = midi_note.round() as i32;

    // Calculate cents deviation from the closest note
    let cents = (midi_note - closest_midi as f32) * 100.0;

    // Get the note name and octave
    let note_index = ((closest_midi % 12) + 12) % 12; // Ensure positive
    let octave = (closest_midi / 12) - 1; // MIDI octave system

    // Format the note with octave and cents deviation
    format!(
        "{}{:+}{:+.0}¢",
        NOTE_NAMES[note_index as usize], octave, cents
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_detection() {
        // Create a sine wave at 440 Hz (A4)
        let sample_rate = 44100;
        let frequency = 440.0;
        let duration = 0.2; // 200ms
        let num_samples = (sample_rate as f32 * duration) as usize;

        let mut samples = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            samples.push((2.0 * PI * frequency * t).sin());
        }

        let detected_pitch = estimate_pitch(&samples, sample_rate);

        assert!(detected_pitch.is_some());
        let pitch = detected_pitch.unwrap();

        // Allow 5% tolerance for pitch detection
        assert!((pitch - 440.0).abs() < 22.0);
    }

    #[test]
    fn test_frequency_to_note() {
        // Test exact frequencies
        assert_eq!(frequency_to_note(440.0), "A4+0¢");
        assert_eq!(frequency_to_note(261.63), "C4+0¢");

        // Test frequencies with cents deviation
        let slightly_sharp_a = 440.0 * 2.0_f32.powf(0.1 / 12.0); // 10 cents sharp
        assert!(frequency_to_note(slightly_sharp_a).contains("+10¢"));

        let slightly_flat_a = 440.0 * 2.0_f32.powf(-0.15 / 12.0); // 15 cents flat
        assert!(frequency_to_note(slightly_flat_a).contains("-15¢"));
    }
}
