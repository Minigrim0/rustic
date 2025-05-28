use super::fft::FrequencyData;
use log::info;

/// Identifies dominant frequencies in the signal
pub fn identify_peaks(
    frequencies: &[FrequencyData],
    threshold: f32,
    min_distance: f32,
) -> Vec<FrequencyData> {
    let mut peaks = Vec::new();

    for i in 1..(frequencies.len() - 1) {
        let prev = &frequencies[i - 1];
        let current = &frequencies[i];
        let next = &frequencies[i + 1];

        // Check if current is a local maximum
        if current.magnitude > prev.magnitude
            && current.magnitude > next.magnitude
            && current.magnitude > threshold
        {
            // Check if it's far enough from previously detected peaks
            if !peaks.iter().any(|peak: &FrequencyData| {
                (peak.frequency - current.frequency).abs() < min_distance
            }) {
                peaks.push(current.clone());
            }
        }
    }

    // Sort peaks by magnitude (descending)
    peaks.sort_by(|a, b| b.magnitude.partial_cmp(&a.magnitude).unwrap());

    info!("Identified {} frequency peaks", peaks.len());
    peaks
}

/// Compute harmonic relationships between frequencies
pub fn analyze_harmonics(peaks: &[FrequencyData], tolerance: f32) -> Vec<Vec<FrequencyData>> {
    if peaks.is_empty() {
        return Vec::new();
    }

    let mut harmonic_series = Vec::new();

    // For each peak, check if it could be a fundamental frequency
    for &fundamental_idx in &[0, 1, 2] {
        if fundamental_idx >= peaks.len() {
            continue;
        }

        let fundamental = &peaks[fundamental_idx];
        let mut harmonics = vec![fundamental.clone()];

        // Check for harmonics (integer multiples of the fundamental)
        for harmonic_number in 2..10 {
            let expected_freq = fundamental.frequency * harmonic_number as f32;
            let allowed_deviation = expected_freq * tolerance;

            // Find the closest peak to the expected harmonic frequency
            if let Some(harmonic) = peaks
                .iter()
                .filter(|p| (p.frequency - expected_freq).abs() <= allowed_deviation)
                .max_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap())
            {
                harmonics.push(harmonic.clone());
            }
        }

        // If we found at least 3 harmonics (including the fundamental), consider it valid
        if harmonics.len() >= 3 {
            harmonic_series.push(harmonics);
        }
    }

    info!("Found {} harmonic series", harmonic_series.len());
    harmonic_series
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_peaks() {
        // Create some sample frequency data with a few peaks
        let mut frequencies = Vec::new();
        for i in 0..100 {
            let freq = i as f32 * 10.0;
            let magnitude = match i {
                10 => 0.8, // peak at 100 Hz
                25 => 1.0, // peak at 250 Hz
                50 => 0.7, // peak at 500 Hz
                _ => 0.1,  // background noise
            };

            frequencies.push(FrequencyData {
                frequency: freq,
                magnitude,
                phase: 0.0,
            });
        }

        let peaks = identify_peaks(&frequencies, 0.5, 50.0);

        assert_eq!(peaks.len(), 3);
        assert_eq!(peaks[0].frequency, 250.0); // Highest peak
        assert_eq!(peaks[1].frequency, 100.0); // Second highest
        assert_eq!(peaks[2].frequency, 500.0); // Third highest
    }

    #[test]
    fn test_analyze_harmonics() {
        // Create peaks at fundamental (100Hz) and harmonics (200Hz, 300Hz, 400Hz)
        let peaks = vec![
            FrequencyData {
                frequency: 100.0,
                magnitude: 1.0,
                phase: 0.0,
            },
            FrequencyData {
                frequency: 198.0,
                magnitude: 0.7,
                phase: 0.0,
            }, // Slightly off 200Hz
            FrequencyData {
                frequency: 302.0,
                magnitude: 0.5,
                phase: 0.0,
            }, // Slightly off 300Hz
            FrequencyData {
                frequency: 400.0,
                magnitude: 0.3,
                phase: 0.0,
            },
            FrequencyData {
                frequency: 150.0,
                magnitude: 0.2,
                phase: 0.0,
            }, // Noise
        ];

        let harmonic_series = analyze_harmonics(&peaks, 0.05); // 5% tolerance

        assert_eq!(harmonic_series.len(), 1);
        assert_eq!(harmonic_series[0].len(), 4); // Fundamental + 3 harmonics
    }
}
