
use music::*;

use tones::{NOTES, TONES_FREQ};
use generator::{Envelope, Generator};
use score::Note;

#[cfg(feature = "plotting")]
use music::plotting::plot_data;

fn main() {
    let mut envelope = Envelope::new(44100.0);

    let scale = 0.2;

    envelope.set_attack(0.1, scale * 1.0, Some((0.1, 0.0)));
    envelope.set_decay(0.05, scale * 0.8, None);
    envelope.set_release(2.0, scale * 0.0, Some((0.5, 0.0)));

    let notes = vec![
        Note::new(TONES_FREQ[NOTES::C as usize][5], 0.0, 0.2),
        Note::new(TONES_FREQ[NOTES::D as usize][5], 0.15, 0.2),
        Note::new(TONES_FREQ[NOTES::DS as usize][5], 0.3, 0.2),

        Note::new(TONES_FREQ[NOTES::DS as usize][3], 0.3, 0.7),
        Note::new(TONES_FREQ[NOTES::DS as usize][4], 0.3, 0.7),

        Note::new(TONES_FREQ[NOTES::DS as usize][5], 1.0, 0.2),
        Note::new(TONES_FREQ[NOTES::F as usize][5], 1.15, 0.2),
        Note::new(TONES_FREQ[NOTES::G as usize][5], 1.3, 0.2),

        Note::new(TONES_FREQ[NOTES::G as usize][3], 1.3, 0.7),
        Note::new(TONES_FREQ[NOTES::G as usize][4], 1.3, 0.7),

        Note::new(TONES_FREQ[NOTES::G as usize][5], 2.0, 0.2),
        Note::new(TONES_FREQ[NOTES::AS as usize][5], 2.15, 0.2),
        Note::new(TONES_FREQ[NOTES::F as usize][5], 2.3, 0.2),

        Note::new(TONES_FREQ[NOTES::F as usize][3], 2.3, 0.7),
        Note::new(TONES_FREQ[NOTES::F as usize][4], 2.3, 0.7),

        Note::new(TONES_FREQ[NOTES::DS as usize][5], 3.0, 0.2),
        Note::new(TONES_FREQ[NOTES::D as usize][5], 3.15, 0.2),
        Note::new(TONES_FREQ[NOTES::C as usize][5], 3.3, 0.2),

        Note::new(TONES_FREQ[NOTES::C as usize][3], 3.3, 0.7),
        Note::new(TONES_FREQ[NOTES::C as usize][4], 3.3, 0.7),


        Note::new(TONES_FREQ[NOTES::C as usize][5], 4.0, 0.2),
        Note::new(TONES_FREQ[NOTES::D as usize][5], 4.15, 0.2),
        Note::new(TONES_FREQ[NOTES::DS as usize][5], 4.3, 0.2),

        Note::new(TONES_FREQ[NOTES::DS as usize][3], 4.3, 0.7),
        Note::new(TONES_FREQ[NOTES::DS as usize][4], 4.3, 0.7),

        Note::new(TONES_FREQ[NOTES::DS as usize][5], 5.0, 0.2),
        Note::new(TONES_FREQ[NOTES::F as usize][5], 5.15, 0.2),
        Note::new(TONES_FREQ[NOTES::G as usize][5], 5.3, 0.2),

        Note::new(TONES_FREQ[NOTES::G as usize][3], 5.3, 0.7),
        Note::new(TONES_FREQ[NOTES::G as usize][4], 5.3, 0.7),

        Note::new(TONES_FREQ[NOTES::G as usize][5], 6.0, 0.2),
        Note::new(TONES_FREQ[NOTES::AS as usize][5], 6.15, 0.2),
        Note::new(TONES_FREQ[NOTES::C as usize][6], 6.3, 0.2),

        Note::new(TONES_FREQ[NOTES::C as usize][4], 6.3, 0.7),
        Note::new(TONES_FREQ[NOTES::C as usize][5], 6.3, 0.7),

        Note::new(TONES_FREQ[NOTES::AS as usize][5], 7.0, 0.2),
        Note::new(TONES_FREQ[NOTES::D as usize][6], 7.15, 0.2),
        Note::new(TONES_FREQ[NOTES::C as usize][6], 7.3, 0.2),

        Note::new(TONES_FREQ[NOTES::C as usize][4], 7.3, 0.7),
        Note::new(TONES_FREQ[NOTES::C as usize][5], 7.3, 0.7),

        Note::new(TONES_FREQ[NOTES::C as usize][6],  8.0, 0.2),
        Note::new(TONES_FREQ[NOTES::D as usize][6],  8.15, 0.2),
        Note::new(TONES_FREQ[NOTES::DS as usize][6], 8.3, 0.2),
        Note::new(TONES_FREQ[NOTES::D as usize][6],  8.6, 0.2),
        Note::new(TONES_FREQ[NOTES::C as usize][6],  8.9, 0.2),
        Note::new(TONES_FREQ[NOTES::AS as usize][5],  9.2, 0.2),
        Note::new(TONES_FREQ[NOTES::GS as usize][5],  9.5, 0.2),
        Note::new(TONES_FREQ[NOTES::G as usize][5],  9.8, 0.2),
        Note::new(TONES_FREQ[NOTES::F as usize][5],  10.1, 0.2),

        Note::new(TONES_FREQ[NOTES::DS as usize][5],  10.7, 0.2),
        Note::new(TONES_FREQ[NOTES::G as usize][5],  10.85, 0.2),
        Note::new(TONES_FREQ[NOTES::F as usize][5],  11.0, 0.2),

        Note::new(TONES_FREQ[NOTES::DS as usize][5],  11.7, 0.2),
        Note::new(TONES_FREQ[NOTES::D as usize][5],  11.85, 0.2),
        Note::new(TONES_FREQ[NOTES::C as usize][5],  12.0, 0.2),

    ];

    let mut notes = notes.iter().map(|n| (n, envelope.attach(&n.generator))).collect::<Vec<(&Note, Generator)>>();

    // On time, off time
    let mut note_status: Vec<(Option<f64>, Option<f64>)> = vec![(None, None); notes.len()];

    let sample_rate = 44100.0;
    let duration = 15.0;

    use rodio::{OutputStream, Sink};
    use rodio::buffer::SamplesBuffer;

    let mut final_vals: Vec<(f32, f32)> = Vec::new();

    // _stream must live as long as the sink
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let mut prev_sample_time = 0.0;

    for over_sample in (0..((sample_rate * duration) as i64)).step_by(sample_rate as usize) {
        let mut vals: Vec<f32> = Vec::new();
        for sample in over_sample..(over_sample + sample_rate as i64) as i64 {
            let current_time = sample as f64 / sample_rate;

            // Check all notes to know their status
            let mut curr_tot = 0.0;
            let mut v = 0;
            for (index, (note, generator)) in notes.iter_mut().enumerate() {
                if current_time >= note.start_time && note_status[index].0.is_none() {
                    note_status[index].0 = Some(current_time as f64);
                }

                if current_time >= note.start_time + note.duration && note_status[index].1.is_none() {
                    note_status[index].1 = Some(current_time as f64);
                }

                let curr: f64;
                (v, curr) = generator.get_at(current_time, note_status[index].0, note_status[index].1);
                curr_tot += curr as f32;
            }
            vals.push(curr_tot);
            if prev_sample_time > current_time {
                println!("Duh {} - {}", prev_sample_time, current_time as f32);
            }
            prev_sample_time = current_time;
            final_vals.push((current_time as f32, curr_tot));
        }
        // println!("{:?}", vals);
        let buff = SamplesBuffer::new(1 as u16, 44100, vals);
        sink.append(buff);
    }

    #[cfg(feature = "plotting")]
    {
        if let Err(e) = plot_data(final_vals, "notes", (-0.1, duration as f32 + 0.1), (-1.1, 1.1), "three_notes.png") {
            println!("Error: {}", e.to_string());
        }
    }

    sink.sleep_until_end();
}
