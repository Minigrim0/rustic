use clap::Parser;
use std::path::PathBuf;

/// Loads mp3, flac, ogg, etc. files into an array of bytes.
mod loader;

/// Performs Fast Fourier Transform on the audio samples.
/// The FFT is used to convert the audio samples from the time domain to the frequency domain.
/// This allows us to analyze the audio samples in terms of their frequency components.
/// Once we have a certain amount of fundamental frequency components, we can track
/// their amplitude and phase over time to extract features from the audio samples.
/// The remaining features can be used to determine noise, pitch, and other characteristics of the audio samples.
mod fft;

/// Contains the functions for calculating the magnitude of the frequencies from
/// the transformed audio samples.
mod magnitudes;

/// Contains the functions for plotting values.
mod plot;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(short, long, default_value = "error")]
    /// Application log level. Accepts trace, debug, info, warn, error
    loglevel: String,

    #[clap(short, long)]
    /// The file to analyze. Can be a mp3, flac, ogg, etc.
    filename: String,

    #[clap(short = 'F', long, default_value = "html")]
    /// The expected output format. Accepts html, json, toml
    format: String,

    #[clap(short, long, default_value = "output")]
    /// The name of the output file.
    output: String,

    #[clap(short, long)]
    /// The path to the configuration file.
    config: Option<String>,

    #[clap(short, long)]
    /// Dump the default config file to stdout.
    dump_config: bool,

    #[clap(short, long, default_value = "true")]
    /// Whether to open a browser window after the analysis. This only applies for html outputs
    pub browser: bool,
}

fn main() {
    colog::init();

    let args = Cli::parse();
    match args.loglevel.as_str() {
        "trace" => log::set_max_level(log::LevelFilter::Trace),
        "debug" => log::set_max_level(log::LevelFilter::Debug),
        "info" => log::set_max_level(log::LevelFilter::Info),
        "warn" => log::set_max_level(log::LevelFilter::Warn),
        "error" => log::set_max_level(log::LevelFilter::Error),
        _ => log::set_max_level(log::LevelFilter::Info),
    }

    let mut sample_loader = loader::Loader::new(&args.filename);

    if let Some(samples) = fft::fft(&mut sample_loader) {
        let data = magnitudes::get_magnitudes(samples.0, samples.1);
        if let Err(e) = plot::plot_freq(&data, &PathBuf::from("Part1.png")) {
            log::error!("Unable to plot frequencies: {}", e);
        }
    }

    if let Some(samples) = fft::fft(&mut sample_loader) {
        let data = magnitudes::get_magnitudes(samples.0, samples.1);
        if let Err(e) = plot::plot_freq(&data, &PathBuf::from("Part2.png")) {
            log::error!("Unable to plot frequencies: {}", e);
        }
    }

    fft::fft(&mut sample_loader);
}
