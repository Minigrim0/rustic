use std::fs::File;
use std::io;
use std::path::Path;

use hound::{SampleFormat, WavReader};
use log::{error, info, warn};
use symphonia::core::audio::Signal;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Audio buffer containing sample data and metadata
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    /// Audio samples (mono, normalized to -1.0 to 1.0 range)
    samples: Vec<f32>,

    /// Sample rate in Hz
    sample_rate: u32,

    /// Duration in seconds
    duration: f32,

    /// Number of channels in the original audio
    channels: u16,
}

impl AudioBuffer {
    /// Create a new audio buffer
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        let duration = samples.len() as f32 / sample_rate as f32;
        Self {
            samples,
            sample_rate,
            duration,
            channels,
        }
    }

    /// Get a reference to the sample data
    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    /// Get the sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the duration in seconds
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Get the number of channels in the original audio
    pub fn channels(&self) -> u16 {
        self.channels
    }
}

/// Audio file loader that supports various formats via Symphonia
pub struct AudioLoader {}

impl AudioLoader {
    /// Create a new audio loader
    pub fn new() -> Self {
        Self {}
    }

    /// Load an audio file from the given path
    pub fn load_file<P: AsRef<Path>>(&self, path: P) -> Result<AudioBuffer, io::Error> {
        let path = path.as_ref();

        // Try to detect the file type from extension
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();

            match ext_str.as_str() {
                "wav" => return self.load_wav(path),
                _ => {
                    // Fall through to generic loader for other formats
                    info!("Using generic loader for file extension: {}", ext_str);
                }
            }
        }

        // Use Symphonia for other formats
        self.load_generic(path)
    }

    /// Load a WAV file using the hound library
    fn load_wav<P: AsRef<Path>>(&self, path: P) -> Result<AudioBuffer, io::Error> {
        info!("Loading WAV file: {}", path.as_ref().display());

        let mut reader = WavReader::open(path).map_err(|e| {
            error!("Failed to open WAV file: {}", e);
            io::Error::other(format!("WAV error: {}", e))
        })?;

        let spec = reader.spec();
        let sample_rate = spec.sample_rate;
        let channels = spec.channels;
        let bits_per_sample: u16 = spec.bits_per_sample;

        info!(
            "WAV file: {} Hz, {} channels, {} bits",
            sample_rate, channels, bits_per_sample
        );

        // Convert samples to f32 and mix down to mono if needed
        let samples = match (spec.sample_format, spec.bits_per_sample) {
            (SampleFormat::Int, 8) => {
                let samples: Vec<i8> = reader.samples().map(|s| s.unwrap_or(0)).collect();
                self.convert_to_mono_f32(samples, channels, |s| s as f32 / 128.0)
            }
            (SampleFormat::Int, 16) => {
                let samples: Vec<i16> = reader.samples().map(|s| s.unwrap_or(0)).collect();
                self.convert_to_mono_f32(samples, channels, |s| s as f32 / 32768.0)
            }
            (SampleFormat::Int, 24) | (SampleFormat::Int, 32) => {
                let samples: Vec<i32> = reader.samples().map(|s| s.unwrap_or(0)).collect();
                self.convert_to_mono_f32(samples, channels, |s| s as f32 / 2147483648.0)
            }
            (SampleFormat::Float, _) => {
                let samples: Vec<f32> = reader.samples().map(|s| s.unwrap_or(0.0)).collect();
                self.convert_to_mono_f32(samples, channels, |s| s)
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!(
                        "Unsupported WAV format: {:?} bits: {}",
                        spec.sample_format, spec.bits_per_sample
                    ),
                ));
            }
        };

        Ok(AudioBuffer::new(samples, sample_rate, channels))
    }

    /// Load any audio format supported by Symphonia
    fn load_generic<P: AsRef<Path>>(&self, path: P) -> Result<AudioBuffer, io::Error> {
        info!(
            "Loading audio file with Symphonia: {}",
            path.as_ref().display()
        );

        // Open the file
        let file = File::open(path.as_ref())?;

        // Create a media source
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // Create a hint to help the format registry
        let mut hint = Hint::new();
        if let Some(ext) = path.as_ref().extension()
            && let Some(ext_str) = ext.to_str()
        {
            hint.with_extension(ext_str);
        }

        // Use the default options
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();
        let decoder_opts = DecoderOptions::default();

        // Probe the media source
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| {
                error!("Error probing audio file: {}", e);
                io::Error::other(format!("Probe error: {}", e))
            })?;

        // Get the format reader
        let mut format = probed.format;

        // Get the default track
        let track = format
            .default_track()
            .ok_or_else(|| io::Error::other("No default track found"))?;

        // Create a decoder for the track
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .map_err(|e| {
                error!("Error creating decoder: {}", e);
                io::Error::other(format!("Decoder error: {}", e))
            })?;

        // Get the sample rate
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track
            .codec_params
            .channels
            .map(|c| c.count() as u16)
            .unwrap_or(2);
        let bits_per_sample: u16 = track.codec_params.bits_per_sample.unwrap_or(16) as u16;
        let track_id = track.id;

        info!(
            "Audio file: {} Hz, {} channels, {} bits",
            sample_rate, channels, bits_per_sample
        );

        // Decode the entire file into a buffer
        let mut sample_buf = Vec::new();

        loop {
            // Get the next packet from the format reader
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::IoError(ref e))
                    if e.kind() == io::ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(e) => {
                    warn!("Error reading packet: {}", e);
                    continue;
                }
            };

            // If the packet is not for the selected track, skip it
            if packet.track_id() != track_id {
                continue;
            }

            // Decode the packet
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // Get the audio buffer
                    let spec = *decoded.spec();
                    let duration = decoded.capacity() as u64;

                    // Convert the audio buffer to samples
                    match decoded {
                        symphonia::core::audio::AudioBufferRef::F32(buf) => {
                            for i in 0..duration {
                                for channel in 0..spec.channels.count() {
                                    if i as usize >= buf.chan(channel).len() {
                                        sample_buf.push(0.0);
                                    } else {
                                        sample_buf.push(buf.chan(channel)[i as usize]);
                                    }
                                }
                            }
                        }
                        symphonia::core::audio::AudioBufferRef::U16(buf) => {
                            for i in 0..duration {
                                for channel in 0..spec.channels.count() {
                                    sample_buf.push(buf.chan(channel)[i as usize] as f32 / 32768.0);
                                }
                            }
                        }
                        symphonia::core::audio::AudioBufferRef::U32(buf) => {
                            for i in 0..duration {
                                for channel in 0..spec.channels.count() {
                                    sample_buf
                                        .push(buf.chan(channel)[i as usize] as f32 / 2147483648.0);
                                }
                            }
                        }
                        symphonia::core::audio::AudioBufferRef::U8(buf) => {
                            for i in 0..duration {
                                for channel in 0..spec.channels.count() {
                                    sample_buf.push(
                                        (buf.chan(channel)[i as usize] as f32 - 128.0) / 128.0,
                                    );
                                }
                            }
                        }
                        _ => {
                            return Err(io::Error::new(
                                io::ErrorKind::Unsupported,
                                "Unsupported sample format",
                            ));
                        }
                    }
                }
                Err(e) => {
                    warn!("Error decoding packet: {}", e);
                    continue;
                }
            }
        }

        // Convert to mono if needed
        let samples = if channels > 1 {
            let mut mono_samples = Vec::with_capacity(sample_buf.len() / channels as usize);
            for i in 0..(sample_buf.len() / channels as usize) {
                let mut sample_sum = 0.0;
                for c in 0..channels {
                    sample_sum += sample_buf[i * channels as usize + c as usize];
                }
                mono_samples.push(sample_sum / channels as f32);
            }
            mono_samples
        } else {
            sample_buf
        };

        Ok(AudioBuffer::new(samples, sample_rate, channels))
    }

    /// Convert multi-channel samples to mono f32 samples
    fn convert_to_mono_f32<T, F>(&self, samples: Vec<T>, channels: u16, convert: F) -> Vec<f32>
    where
        F: Fn(T) -> f32,
        T: Copy,
    {
        if channels == 1 {
            // Already mono, just convert
            samples.into_iter().map(convert).collect()
        } else {
            // Mix down to mono
            let samples_per_channel = samples.len() / channels as usize;
            let mut mono_samples = Vec::with_capacity(samples_per_channel);

            for i in 0..samples_per_channel {
                let mut sample_sum = 0.0;
                for c in 0..channels {
                    let sample = samples[i * channels as usize + c as usize];
                    sample_sum += convert(sample);
                }
                mono_samples.push(sample_sum / channels as f32);
            }

            mono_samples
        }
    }
}

impl Default for AudioLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::{WavSpec, WavWriter};
    use std::io::Cursor;

    fn create_test_wav() -> Vec<u8> {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut buffer = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut buffer, spec).unwrap();
            // Generate a 440 Hz sine wave for 0.1 seconds
            for t in 0..4410 {
                let sample = (t as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin();
                writer.write_sample((sample * 32767.0) as i16).unwrap();
            }
            writer.finalize().unwrap();
        }

        buffer.into_inner()
    }

    #[test]
    fn test_load_wav_from_memory() {
        let wav_data = create_test_wav();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("test.wav");

        std::fs::write(&temp_path, wav_data).unwrap();

        let loader = AudioLoader::new();
        let result = loader.load_file(&temp_path);

        assert!(result.is_ok());
        let buffer = result.unwrap();

        assert_eq!(buffer.sample_rate(), 44100);
        assert_eq!(buffer.channels(), 1);
        assert_eq!(buffer.samples().len(), 4410);
    }
}
