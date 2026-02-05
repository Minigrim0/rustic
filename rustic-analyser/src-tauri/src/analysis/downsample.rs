/// Downsample a waveform for display by emitting min/max pairs per bucket.
///
/// If `samples.len() <= target_points`, the original samples are returned unchanged.
/// Otherwise the samples are split into `target_points / 2` buckets and each bucket
/// contributes its `[min, max]` pair, producing at most `target_points` values.
///
/// Returns `(output_samples, was_downsampled)`.
pub fn downsample_waveform(samples: &[f32], target_points: u32) -> (Vec<f32>, bool) {
    let target = target_points as usize;

    if samples.len() <= target {
        return (samples.to_vec(), false);
    }

    let num_buckets = target / 2;
    if num_buckets == 0 {
        return (vec![], true);
    }

    let bucket_size = samples.len() as f64 / num_buckets as f64;
    let mut out = Vec::with_capacity(num_buckets * 2);

    for i in 0..num_buckets {
        let start = (i as f64 * bucket_size) as usize;
        let end = (((i + 1) as f64 * bucket_size) as usize).min(samples.len());
        let bucket = &samples[start..end];

        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for &s in bucket {
            if s < min {
                min = s;
            }
            if s > max {
                max = s;
            }
        }
        out.push(min);
        out.push(max);
    }

    (out, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_downsample_when_short() {
        let samples = vec![0.0, 0.5, -0.5, 1.0];
        let (out, downsampled) = downsample_waveform(&samples, 10);
        assert!(!downsampled);
        assert_eq!(out, samples);
    }

    #[test]
    fn no_downsample_when_exact() {
        let samples = vec![0.0; 100];
        let (out, downsampled) = downsample_waveform(&samples, 100);
        assert!(!downsampled);
        assert_eq!(out.len(), 100);
    }

    #[test]
    fn downsamples_correctly() {
        // 100 samples → target 10 → 5 buckets of 20 samples each → 10 output values
        let mut samples = vec![0.0f32; 100];
        // Bucket 0 (0..20): put known extremes
        samples[5] = -1.0;
        samples[10] = 1.0;
        // Bucket 1 (20..40): all zeros → min=0, max=0
        // Bucket 2 (40..60): small values
        samples[50] = 0.25;

        let (out, downsampled) = downsample_waveform(&samples, 10);
        assert!(downsampled);
        assert_eq!(out.len(), 10);

        // Bucket 0: min=-1.0, max=1.0
        assert_eq!(out[0], -1.0);
        assert_eq!(out[1], 1.0);

        // Bucket 1: min=0.0, max=0.0
        assert_eq!(out[2], 0.0);
        assert_eq!(out[3], 0.0);

        // Bucket 2: min=0.0, max=0.25
        assert_eq!(out[4], 0.0);
        assert_eq!(out[5], 0.25);
    }

    #[test]
    fn zero_target_returns_empty() {
        let samples = vec![1.0; 10];
        let (out, downsampled) = downsample_waveform(&samples, 0);
        assert!(downsampled);
        assert!(out.is_empty());
    }
}
