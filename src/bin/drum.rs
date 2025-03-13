use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use log::error;

use rustic::instruments::prelude::{HiHat, Kick};
use rustic::instruments::Instrument;
use rustic::prelude::App;
use rustic::Note;

fn main() {
    colog::init();

    let app = App::init();

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
    let mut complete_value_list = vec![];
    for i in 0..20 {
        values.clear();

        kick.start_note(Note(rustic::core::tones::NOTES::A, 0), 0.0);
        hihat.start_note(Note(rustic::core::tones::NOTES::A, 0), 0.0);

        for _ in 0..(app.config.system.sample_rate as usize / 2) {
            kick.tick();
            if i % 2 == 0 {
                values.push(kick.get_output());
                values.push(0.0);

                complete_value_list.push(kick.get_output());
                complete_value_list.push(0.0);
            } else {
                values.push(0.0);
                values.push(kick.get_output());
                complete_value_list.push(0.0);
                complete_value_list.push(kick.get_output())
            }
        }

        kick.stop_note(Note(rustic::core::tones::NOTES::A, 0));
        hihat.stop_note(Note(rustic::core::tones::NOTES::A, 0));

        sink.append(SamplesBuffer::new(
            2 as u16,
            app.config.system.sample_rate as u32,
            values.iter().map(|n| *n).collect::<Vec<f32>>(),
        ));
    }

    #[cfg(feature = "plotting")]
    {
        use rustic::plotting::Plot;

        let left_ear = complete_value_list
            .iter()
            .enumerate()
            .filter_map(|(position, element)| {
                if position % 2 == 0 {
                    Some((
                        (position as f32 / 2.0) / app.config.system.sample_rate,
                        *element,
                    ))
                } else {
                    None
                }
            })
            .collect();

        let right_ear = complete_value_list
            .iter()
            .enumerate()
            .filter_map(|(position, element)| {
                if position % 2 == 1 {
                    Some((
                        (position as f32 / 2.0) / app.config.system.sample_rate,
                        *element,
                    ))
                } else {
                    None
                }
            })
            .collect();

        let mut plot = Plot::new("Simple Drum", (-0.1, 1.0), (-1.1, 1.1), "drum.png")?;
        plot.plot(left_ear, "left ear");
        plot.plt(right_ear, "right_ear");
        plot.render();
    }

    sink.sleep_until_end();
}
