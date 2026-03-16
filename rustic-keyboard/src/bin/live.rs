use std::io::{self, Write};

use rustic::audio::{EventFilter, LogConfig};
use rustic::core::envelope::prelude::{ADSREnvelopeBuilder, LinearSegment};
use rustic::instruments::prelude::KeyboardBuilder;
use rustic::prelude::App;

use rustic_keyboard::KeyboardPlayer;
use rustic_keyboard::inputs::{InputDevice, list_input_devices};

fn main() {
    let _ = rustic::init_logging(&LogConfig::default(), std::path::Path::new("."));

    // — Device selection —————————————————————————————————————————————————————
    let devices = list_input_devices();

    if devices.is_empty() {
        eprintln!("No input devices found under /dev/input.");
        std::process::exit(1);
    }

    println!("Available input devices:");
    for (i, dev) in devices.iter().enumerate() {
        let marker = if dev.is_available() { " " } else { "!" };
        println!("  [{i}] [{marker}] {}", dev.label());
    }
    println!("  (!) = not accessible, run with sudo or add yourself to the `input` group");

    let chosen = loop {
        print!("Select device [0-{}]: ", devices.len() - 1);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        match input.trim().parse::<usize>() {
            Ok(n) if n < devices.len() => break n,
            _ => println!(
                "  Please enter a number between 0 and {}.",
                devices.len() - 1
            ),
        }
    };

    let device = match devices.into_iter().nth(chosen).unwrap() {
        InputDevice::Available { label, device } => {
            println!("Using: {label}");
            device
        }
        InputDevice::Inaccessible { label } => {
            eprintln!("Cannot open '{label}'.");
            eprintln!("Try: sudo usermod -aG input $USER  (then log out and back in)");
            std::process::exit(1);
        }
    };

    // — Audio setup ——————————————————————————————————————————————————————————
    let mut app = App::new();

    let keyboard = KeyboardBuilder::new()
        .with_voices(8)
        .with_note_envelope(
            ADSREnvelopeBuilder::new()
                .attack(Box::new(LinearSegment::new(0.0, 1.0, 0.005))) // 5 ms
                .decay(Box::new(LinearSegment::new(1.0, 1.0, 0.001))) // 1 ms, no drop → sustain at 1.0
                .release(Box::new(LinearSegment::new(1.0, 0.0, 0.2))) // 200 ms
                .build(),
        )
        .build();

    app.add_instrument(Box::new(keyboard));
    app.start(EventFilter::default())
        .expect("Failed to start audio engine");

    println!("Q-P → octave 5 (C–A)   |   A-L → octave 4 (C–G#)   |   Ctrl+C to quit");

    KeyboardPlayer::new().run_with_device(device, &app);
}
