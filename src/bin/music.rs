use threadpool::ThreadPool;

use music::*;

use music::sine_wave::SineWave;
use music::low_pass_filter::LowPassFilter;
use music::sound_system::SoundSystem;
use music::adsr::ADSR;
use music::player::Player;
use music::score::{Note, Score};



fn main() {
    let adsr = ADSR::new(0.1, 0.1, 0.7, 0.3, 44100.0);
    let sine_wave = SineWave::new(440.0, 1.0, adsr);
    let low_pass = LowPassFilter::new(200.0);

    let mut sound_system = SoundSystem::new(Box::new(sine_wave));
    sound_system.add_filter(Box::new(low_pass));

    let mut player = Player::new(sound_system, 44100);

    let notes = vec![
        Note { frequency: 440.0, start_time: 0.0, duration: 1.0 },
        Note { frequency: 494.0, start_time: 1.0, duration: 1.0 },
        Note { frequency: 523.0, start_time: 2.0, duration: 1.0 },
    ];

    let score = Score::new(notes);
    score.play(&mut player);
}