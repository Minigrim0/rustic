use super::FrequencyData;

/// Pick the top N frequency peaks from a spectrum, ensuring a minimum Hz
/// separation between selected peaks to avoid picking adjacent FFT bins
/// that belong to the same spectral feature.
pub fn pick_top_frequencies(
    frequencies: &[FrequencyData],
    count: usize,
    min_distance_hz: f32,
) -> Vec<FrequencyData> {
    if frequencies.is_empty() {
        return Vec::new();
    }

    // Sort indices by magnitude descending
    let mut indices: Vec<usize> = (0..frequencies.len()).collect();
    indices.sort_unstable_by(|&a, &b| {
        frequencies[b]
            .magnitude
            .partial_cmp(&frequencies[a].magnitude)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut picked = Vec::with_capacity(count);

    for &idx in &indices {
        if picked.len() >= count {
            break;
        }
        let candidate = &frequencies[idx];
        let too_close = picked
            .iter()
            .any(|p: &FrequencyData| (p.frequency - candidate.frequency).abs() < min_distance_hz);
        if !too_close {
            picked.push(candidate.clone());
        }
    }

    picked
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_freq(frequency: f32, magnitude: f32) -> FrequencyData {
        FrequencyData {
            frequency,
            magnitude,
            phase: 0.0,
        }
    }

    #[test]
    fn test_pick_top_frequencies_basic() {
        let freqs = vec![
            make_freq(100.0, 0.5),
            make_freq(200.0, 0.9),
            make_freq(210.0, 0.85), // too close to 200
            make_freq(400.0, 0.7),
            make_freq(800.0, 0.3),
        ];

        let top = pick_top_frequencies(&freqs, 3, 50.0);
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].frequency, 200.0); // highest magnitude
        assert_eq!(top[1].frequency, 400.0); // next highest (210 skipped)
        assert_eq!(top[2].frequency, 100.0);
    }

    #[test]
    fn test_pick_top_frequencies_empty() {
        let top = pick_top_frequencies(&[], 10, 50.0);
        assert!(top.is_empty());
    }

    #[test]
    fn test_pick_top_frequencies_fewer_than_count() {
        let freqs = vec![make_freq(100.0, 0.5), make_freq(500.0, 0.8)];
        let top = pick_top_frequencies(&freqs, 10, 50.0);
        assert_eq!(top.len(), 2);
    }
}
