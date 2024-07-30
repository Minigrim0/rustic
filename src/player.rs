// player.rs
use rodio::{OutputStream, Sink, Source};
use std::time::Duration;
use std::iter::Iterator;
use crate::sound_system::SoundSystem;


pub struct SoundSource {
    pub sound_system: SoundSystem,
    sample_rate: u32,
    duration: Duration,
    current_sample: u32,
}

impl SoundSource {
    pub fn new(sound_system: SoundSystem, sample_rate: u32, duration: Duration) -> Self {
        Self {
            sound_system,
            sample_rate,
            duration,
            current_sample: 0,
        }
    }
}

impl Iterator for SoundSource {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= (self.duration.as_secs_f32() * self.sample_rate as f32) as u32 {
            return None;
        }
        let time = self.current_sample as f32 / self.sample_rate as f32;
        let sample = self.sound_system.generate_sample(time);

        // Convert f32 sample to i16
        let sample_i16 = (sample * i16::MAX as f32) as i16;
        self.current_sample += 1;

        Some(sample_i16)
    }
}

impl Source for SoundSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(self.duration)
    }
}

pub struct Player {
    pub sound_system: SoundSystem,
    sample_rate: u32,
}

impl Player {
    pub fn new(sound_system: SoundSystem, sample_rate: u32) -> Self {
        Self {
            sound_system,
            sample_rate,
        }
    }

    pub fn play(&mut self, duration: f32) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let source: SoundSource = SoundSource::new(self.sound_system, self.sample_rate, Duration::from_secs_f32(duration));
        sink.append(source);

        sink.sleep_until_end();
    }
}
