use std::collections::BinaryHeap;

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use rustic::generator::{Envelope, GENERATORS};
use rustic::score::Note;
use rustic::tones::{NOTES, TONES_FREQ};

#[cfg(feature = "plotting")]
use rustic::plotting::plot_data;

fn main() {
    let scale = 0.2; // Master volume
    let duration = 14.0; // Duration of the song
    let sample_rate = 44100.0; // Sample rate
    let mut event_queue = BinaryHeap::new(); // Event queue for the notes to play

    let envelope = {
        let mut env = Envelope::new();
        env.set_attack(0.1, scale * 1.0, Some((0.1, 0.0)));
        env.set_decay(0.05, scale * 0.8, None);
        env.set_release(1.5, scale * 0.0, Some((0.5, 0.0)));
        env
    };

    let other_envelope = {
        let mut env = Envelope::new();
        env.set_attack(0.0, scale * 1.0, None);
        env.set_decay(0.05, scale * 0.0, None);
        env.set_release(0.0, scale * 0.0, None);
        env
    };

    let notes = {
        let mut perc: Vec<Note> = Vec::new();
        for i in 0..(duration as i32) + 1 {
            perc.push(
                Note::new(TONES_FREQ[NOTES::C as usize][3], i as f32 + 0.75, 0.1)
                    .with_generator(GENERATORS::NOISE),
            );
            perc.push(
                Note::new(TONES_FREQ[NOTES::C as usize][3], i as f32 + 0.9, 0.1)
                    .with_generator(GENERATORS::NOISE),
            );
        }

        let notes = vec![
            Note::new(TONES_FREQ[NOTES::C as usize][5], 0.0, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::D as usize][5], 0.15, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::DS as usize][5], 0.3, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::DS as usize][3], 0.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::DS as usize][4], 0.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::DS as usize][5], 1.0, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::F as usize][5], 1.15, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::G as usize][5], 1.3, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::G as usize][3], 1.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::G as usize][4], 1.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::G as usize][5], 2.0, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::AS as usize][5], 2.15, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::F as usize][5], 2.3, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::F as usize][3], 2.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::F as usize][4], 2.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::DS as usize][5], 3.0, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::D as usize][5], 3.15, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][5], 3.3, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][3], 3.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][4], 3.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][5], 4.0, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::D as usize][5], 4.15, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::DS as usize][5], 4.3, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::DS as usize][3], 4.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::DS as usize][4], 4.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::DS as usize][5], 5.0, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::F as usize][5], 5.15, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::G as usize][5], 5.3, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::G as usize][3], 5.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::G as usize][4], 5.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::G as usize][5], 6.0, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::AS as usize][5], 6.15, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][6], 6.3, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][4], 6.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][5], 6.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::AS as usize][5], 7.0, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::D as usize][6], 7.15, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][6], 7.3, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][4], 7.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][5], 7.3, 0.7).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][6], 8.0, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::D as usize][6], 8.15, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::DS as usize][6], 8.3, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::D as usize][6], 8.6, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][6], 8.9, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::AS as usize][5], 9.2, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::GS as usize][5], 9.5, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::G as usize][5], 9.8, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::F as usize][5], 10.1, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::DS as usize][5], 10.7, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::G as usize][5], 10.85, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::F as usize][5], 11.0, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::DS as usize][5], 11.7, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::D as usize][5], 11.85, 0.2).with_generator(GENERATORS::SAW),
            Note::new(TONES_FREQ[NOTES::C as usize][5], 12.0, 0.2).with_generator(GENERATORS::SAW),
        ];

        let mut enveloped_notes = notes
            .into_iter()
            .map(|n| n.with_envelope(&envelope))
            .collect::<Vec<Note>>();
        enveloped_notes.append(
            perc.into_iter()
                .map(|n| n.with_envelope(&other_envelope))
                .collect::<Vec<Note>>()
                .as_mut(),
        );
        enveloped_notes
    };

    for note in notes.into_iter() {
        event_queue.push(note);
    }

    let mut final_vals: Vec<(f32, f32)> = Vec::new();
    let mut vals: Vec<(f32, f32)> = Vec::new();
    let mut current_notes: Vec<Note> = Vec::new();
    let mut current_time: f32 = 0.0;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    'builder: loop {
        current_time += 1.0 / sample_rate as f32;

        // Find currently playing notes in the binary heap and push themm to the current_notes vector
        'fetcher: loop {
            if let Some(note) = event_queue.peek() {
                if note.start_time <= current_time {
                    current_notes.push(event_queue.pop().unwrap());
                } else {
                    break 'fetcher;
                }
            }
            break 'fetcher;
        }

        // Push to sink every second
        if vals.len() == sample_rate as usize {
            println!("{} - Pushing to sync !", current_time);
            sink.append(SamplesBuffer::new(
                1 as u16,
                44100,
                vals.iter().map(|(_, n)| *n).collect::<Vec<f32>>(),
            ));
            final_vals.append(&mut vals);
        }

        current_notes.retain(|n| !n.is_completed(current_time));

        // If there are no notes to play, break the loop
        if current_notes.is_empty() {
            if event_queue.is_empty() {
                sink.append(SamplesBuffer::new(
                    1 as u16,
                    44100,
                    vals.iter().map(|(_, n)| *n).collect::<Vec<f32>>(),
                ));
                final_vals.append(&mut vals);
                println!(
                    "Event queue is empty ! Breaking the loop - {}",
                    current_time
                );
                break 'builder;
            }
            vals.push((current_time, 0.0));
            continue;
        }

        // Generate the current sample
        vals.push((
            current_time,
            current_notes
                .iter_mut()
                .map(|note| note.get_at(current_time))
                .sum(),
        ));
    }
    println!("Done");

    // _stream must live as long as the sink

    // for over_sample in (0..((sample_rate * duration) as i64)).step_by(sample_rate as usize) {
    //     let mut vals: Vec<f32> = Vec::new();
    //     println!("Generating buffer");
    //     let start = Instant::now();
    //     for sample in over_sample..(over_sample + sample_rate as i64) as i64 {
    //         let current_time = sample as f32 / sample_rate;

    //         // Check all notes to know their status
    //         let curr_tot = notes.iter_mut().enumerate().map(|(index, (note, generator))| {
    //             if current_time >= note.start_time && note_status[index].0.is_none() {
    //                 note_status[index].0 = Some(current_time as f32);
    //             }

    //             if current_time >= note.start_time + note.duration && note_status[index].1.is_none() {
    //                 note_status[index].1 = Some(current_time as f32);
    //             }

    //             let curr = generator.get_at(current_time, note_status[index].0, note_status[index].1);
    //             curr as f32
    //         }).sum();

    //         vals.push(curr_tot);

    //         final_vals.push((current_time as f32, curr_tot));
    //     }
    //     // println!("{:?}", vals);
    //     println!("Pushing buffer ({:?} s)", start.elapsed());
    //     let buff = SamplesBuffer::new(1 as u16, 44100, vals);
    //     sink.append(buff);
    // }
    // println!("Done");

    #[cfg(feature = "plotting")]
    {
        if let Err(e) = plot_data(
            final_vals,
            "notes",
            (0.0, duration as f32 + 0.1),
            (-1.1, 1.1),
            "three_notes.png",
        ) {
            println!("Error: {}", e.to_string());
        }
    }

    sink.sleep_until_end();
}
