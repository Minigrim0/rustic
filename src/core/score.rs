use std::collections::BinaryHeap;

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use log::info;

use super::note::Note;

#[cfg(feature = "plotting")]
use crate::plotting::plot_data;

pub struct Score {
    notes: BinaryHeap<Note>,
    playing: bool,
    current_sample: i32,
    sample_rate: i32,
    name: String,
}

impl Score {
    pub fn new(name: String, sample_rate: i32) -> Self {
        Self {
            notes: BinaryHeap::new(),
            playing: false,
            current_sample: 0,
            sample_rate,
            name,
        }
    }

    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    pub fn play(&mut self) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let mut current_notes = vec![];
        let mut vals = vec![];

        #[cfg(feature = "plotting")]
        let mut plotting_vals = vec![];

        'builder: loop {
            self.current_sample += 1;
            let current_time = self.current_sample as f32 / self.sample_rate as f32;

            // Find currently playing notes in the binary heap and push themm to the current_notes vector
            'fetcher: loop {
                if let Some(note) = self.notes.peek() {
                    if note.start_time <= current_time {
                        current_notes.push(self.notes.pop().unwrap());
                    } else {
                        break 'fetcher;
                    }
                }
                break 'fetcher;
            }

            // Push to sink every second
            if vals.len() == self.sample_rate as usize {
                sink.append(SamplesBuffer::new(
                    1 as u16,
                    self.sample_rate as u32,
                    vals.iter().map(|(_, n)| *n).collect::<Vec<f32>>(),
                ));
                #[cfg(feature = "plotting")]
                plotting_vals.append(&mut vals);
            }

            current_notes.retain(|n| !n.is_completed(current_time));

            // If there are no notes to play, break the loop
            if current_notes.is_empty() {
                if self.notes.is_empty() {
                    sink.append(SamplesBuffer::new(
                        1 as u16,
                        self.sample_rate as u32,
                        vals.iter().map(|(_, n)| *n).collect::<Vec<f32>>(),
                    ));

                    #[cfg(feature = "plotting")]
                    plotting_vals.append(&mut vals);

                    info!(
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
                    .map(|note| note.tick(self.current_sample, self.sample_rate as i32))
                    .sum(),
            ));
        }

        #[cfg(feature = "plotting")]
        {
            let duration = plotting_vals.len() as f32 / self.sample_rate as f32 + 0.1;
            if let Err(e) = plot_data(
                plotting_vals,
                format!("Score {} (SR={}Hz)", self.name, self.sample_rate).as_str(),
                (0.0, duration),
                (-1.1, 1.1),
                format!("score_{}_{}.png", self.name, self.sample_rate).as_str(),
            ) {
                println!("Error: {}", e.to_string());
            }
        }

        sink.sleep_until_end();
    }
}
