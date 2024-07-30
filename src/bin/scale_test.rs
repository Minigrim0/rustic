use std::collections::HashMap;
use std::io::{self, Read};
use music::inputs;
use music::tones::{NOTES, TONES_FREQ};


fn main() -> io::Result<()>{
    let mut buf = [0u8; 1024];
    let mut stdin = io::stdin();

    inputs::raw_terminal::setup_raw_terminal()?;

    let mappings: HashMap<u8, NOTES> = HashMap::from([
            (b'q', NOTES::C),
            (b'w', NOTES::CS),
            (b'e', NOTES::D),
            (b'r', NOTES::DS),
            (b't', NOTES::E),
            (b'y', NOTES::F),
            (b'u', NOTES::FS),
            (b'i', NOTES::G),
            (b'o', NOTES::GS),
            (b'p', NOTES::A),
            (b'[', NOTES::AS),
            (b']', NOTES::B),
        ]
    );

    let current_scale = 4;

    loop {
        let size = stdin.read(&mut buf)?;
        let data = &buf[0..size];

        println!("stdin data: {:?}", data);
        println!("Tone: {}", TONES_FREQ[
            mappings.get(&data[0]).unwrap_or(&NOTES::C).clone() as usize
        ][
            current_scale
        ]);
    }
}
