use std::fs::File;
use std::path::Path;

use log::{error, info};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct Loader {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
}

impl Loader {
    pub fn new<S: AsRef<str>>(filename: S) -> Self {
        let file = Box::new(File::open(Path::new(filename.as_ref())).unwrap());

        // Create the media source stream using the boxed media source from above.
        let mss = MediaSourceStream::new(file, Default::default());

        let extension = Path::new(filename.as_ref())
            .extension()
            .unwrap()
            .to_str()
            .unwrap();

        // Create a hint to help the format registry guess what format reader is appropriate. In this
        // example we'll leave it empty.
        let mut hint = Hint::new();
        hint.with_extension(extension);

        // Use the default options when reading and decoding.
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();

        // Probe the media source stream for a format.
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .unwrap();

        // Get the format reader yielded by the probe operation.
        let format = probed.format;

        let track = format.default_track().unwrap();
        let track_id = track.id;

        // Create a decoder for the track.
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .unwrap();

        Self {
            format,
            decoder,
            track_id,
        }
    }

    /// Get the next packet from the format reader. If no
    /// packet is available, return None.
    /// The return value is a tuple containing the sample buffer and the sample rate.
    pub fn next(&mut self) -> Option<(SampleBuffer<f32>, u32)> {
        let packet = match self.format.next_packet() {
            Ok(pack) => pack,
            Err(e) => {
                info!("Unable to get next packet: {}", e);
                return None;
            }
        };

        // If the packet does not belong to the selected track, skip it.
        if packet.track_id() != self.track_id {
            info!("Skipping packet from track {}", packet.track_id());
            return None;
        }

        // Decode the packet into audio samples.
        match self.decoder.decode(&packet) {
            Ok(audio_buf) => {
                let sample_rate: u32 = audio_buf.spec().rate;
                info!("Decoded packet with {} samples", audio_buf.capacity());
                let spec = *audio_buf.spec();
                let duration = audio_buf.capacity() as u64;
                let mut sample_buf = SampleBuffer::<f32>::new(duration, spec);
                info!("Copying samples to buffer");
                sample_buf.copy_interleaved_ref(audio_buf);
                info!(
                    "Returning sample buffer with {} samples",
                    sample_buf.samples().len()
                );
                // for sample in sample_buf.samples().iter() {
                //     info!("Sample: {}", sample);
                // }
                Some((sample_buf, sample_rate))
            }
            Err(Error::DecodeError(e)) => {
                error!("Unable to decode packet: {}", e);
                None
            }
            Err(_) => {
                error!("Unknown error decoding packet");
                None
            }
        }
    }
}
