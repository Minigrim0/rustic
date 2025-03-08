use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use log::error;

use rustic::instruments::prelude::{HiHat, Kick};
use rustic::instruments::Instrument;
use rustic::Note;

fn main() {
    colog::init();
    // let master_volume = 0.2;
    let sample_rate = 44100.0; // 44100 Hz

    let mut hihat = match HiHat::new() {
        Ok(h) => h,
        Err(e) => {
            error!("Unable to create hihat: {}", e);
            return;
        }
    };

    let mut kick = Kick::new();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let mut values = vec![];
    for _ in 0..10 {
        values.clear();

        hihat.start_note(Note(rustic::core::tones::NOTES::A, 0), 0.0);
        kick.start_note(Note(rustic::core::tones::NOTES::A, 0), 0.0);

        for _ in 0..(sample_rate as usize / 2) {
            hihat.tick();
            kick.tick();
            values.push(kick.get_output());
        }

        hihat.stop_note(Note(rustic::core::tones::NOTES::A, 0));
        kick.stop_note(Note(rustic::core::tones::NOTES::A, 0));

        sink.append(SamplesBuffer::new(
            1 as u16,
            sample_rate as u32,
            values.iter().map(|n| *n).collect::<Vec<f32>>(),
        ));
    }

    sink.sleep_until_end();
}
