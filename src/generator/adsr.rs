// ADSR - Attack, Decay, Sustain, Release
// This is a simple ADSR envelope generator that can be used to control the amplitude of a sound over time.
pub struct ADSR {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    sample_rate: f32,
}

impl ADSR {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32, sample_rate: f32) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
            sample_rate,
        }
    }

    /**
        * Get the amplitude of the envelope at a given time.
        * @param time The current time in seconds.
        * @param note_on_time The time the note was pressed in seconds.
        * @param note_off_time The time the note was released in seconds.
        * @return The amplitude of the envelope at the given time.
    */
    pub fn get_amplitude(&self, time: f32, note_on_time: f32, note_off_time: f32) -> f32 {
        let mut amplitude = 0.0;
        let elapsed = time - note_on_time;

        if elapsed < self.attack {
            amplitude = elapsed / self.attack;
        } else if elapsed < self.attack + self.decay {
            amplitude = 1.0 - (elapsed - self.attack) * (1.0 - self.sustain) / self.decay;
        } else if note_off_time > 0.0 && time > note_off_time {
            let release_time = time - note_off_time;
            amplitude = self.sustain * (1.0 - release_time / self.release);
        } else {
            amplitude = self.sustain;
        }

        amplitude.max(0.0)
    }
}
