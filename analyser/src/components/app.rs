use js_sys::{ArrayBuffer, Uint8Array};
use log::{error, info};
use tauri_sys::core as tauri;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use wasm_bindgen_futures::spawn_local;
use web_sys::File;
use yew::prelude::*;

use crate::components::audio_visualizer::AudioVisualizer;
use crate::components::file_upload::FileUpload;
use crate::components::frequency_chart::FrequencyChart;
use crate::components::spectrum_display::SpectrumDisplay;

#[derive(Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub name: String,
    pub duration: f32,
}

pub enum Msg {
    FileSelected(File),
    FileLoaded(AudioData),
    AnalysisComplete(AnalysisResult),
    ProcessingError(String),
    ClearData,
}

pub struct App {
    audio_data: Option<AudioData>,
    analysis_result: Option<AnalysisResult>,
    error_message: Option<String>,
    is_processing: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AnalysisResult {
    pub frequencies: Vec<(f32, f32)>,
    pub spectrogram: Vec<Vec<f32>>,
    pub peak_frequency: f32,
    pub pitch: Option<f32>,
    pub note: Option<String>,
    pub rms_level: f32,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            audio_data: None,
            analysis_result: None,
            error_message: None,
            is_processing: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FileSelected(file) => {
                self.is_processing = true;
                self.error_message = None;
                let link = ctx.link().clone();

                spawn_local(async move {
                    info!("Starting file processing for {}", file.name());
                    match process_file(file).await {
                        Ok(data) => {
                            info!("File processed successfully");
                            link.send_message(Msg::FileLoaded(data));
                        }
                        Err(err) => {
                            error!("Error processing file: {}", err);
                            link.send_message(Msg::ProcessingError(err));
                        }
                    }
                });

                true
            }
            Msg::FileLoaded(data) => {
                info!(
                    "File loaded: {}, sample rate: {}",
                    data.name, data.sample_rate
                );
                self.audio_data = Some(data.clone());

                // Now run analysis on the backend
                let link = ctx.link().clone();
                self.is_processing = true;

                spawn_local(async move {
                    match analyze_audio_data(&data).await {
                        Ok(result) => {
                            link.send_message(Msg::AnalysisComplete(result));
                        }
                        Err(err) => {
                            link.send_message(Msg::ProcessingError(err));
                        }
                    }
                });

                true
            }
            Msg::AnalysisComplete(result) => {
                info!(
                    "Analysis complete: peak frequency: {}",
                    result.peak_frequency
                );
                self.analysis_result = Some(result);
                self.is_processing = false;
                true
            }
            Msg::ProcessingError(message) => {
                error!("Processing error: {}", message);
                self.error_message = Some(message);
                self.is_processing = false;
                true
            }
            Msg::ClearData => {
                self.audio_data = None;
                self.analysis_result = None;
                self.error_message = None;
                self.is_processing = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="app-container">
                <header>
                    <h1>{ "Sample Analyser" }</h1>
                    <p>{ "Upload audio files for frequency analysis" }</p>
                </header>

                <main>
                    <FileUpload on_file_selected={ctx.link().callback(Msg::FileSelected)} />

                    { self.view_error() }

                    { if self.is_processing {
                        html! { <div class="loading"><p>{ "Processing..." }</p></div> }
                    } else {
                        html! {}
                    }}

                    { self.view_analysis(ctx) }
                </main>

                <footer>
                    <p>{ "© 2024 Rustic Sample Analyser" }</p>
                </footer>
            </div>
        }
    }
}

impl App {
    fn view_error(&self) -> Html {
        if let Some(error) = &self.error_message {
            html! {
                <div class="error-message">
                    <p>{ format!("Error: {}", error) }</p>
                </div>
            }
        } else {
            html! {}
        }
    }

    fn view_analysis(&self, ctx: &Context<Self>) -> Html {
        if let (Some(data), Some(analysis)) = (&self.audio_data, &self.analysis_result) {
            html! {
                <div class="analysis-container">
                    <h2>{ format!("Analysis for: {}", data.name) }</h2>

                    <div class="analysis-info">
                        <p>{ format!("Duration: {:.2} seconds", data.duration) }</p>
                        <p>{ format!("Sample Rate: {} Hz", data.sample_rate) }</p>
                        <p>{ format!("Peak Frequency: {:.2} Hz", analysis.peak_frequency) }</p>
                        { if let Some(pitch) = analysis.pitch {
                            html! { <p>{ format!("Estimated Pitch: {:.2} Hz ({})",
                                pitch, analysis.note.clone().unwrap_or_default()) }</p> }
                        } else {
                            html! {}
                        }}
                    </div>

                    <div class="visualizations">
                        <div class="visualization-card">
                            <h3>{ "Waveform" }</h3>
                            <AudioVisualizer samples={data.samples.clone()} sample_rate={data.sample_rate} />
                        </div>

                        <div class="visualization-card">
                            <h3>{ "Frequency Spectrum" }</h3>
                            <FrequencyChart
                                frequencies={analysis.frequencies.clone()}
                                sample_rate={data.sample_rate}
                            />
                        </div>

                        <div class="visualization-card">
                            <h3>{ "Spectrogram" }</h3>
                            <SpectrumDisplay
                                spectrogram={analysis.spectrogram.clone()}
                                min_frequency={Some(20.0)}
                                max_frequency={Some(20000.0)}
                                sample_rate={data.sample_rate}
                            />
                        </div>
                    </div>

                    <button
                        onclick={ctx.link().callback(|_| Msg::ClearData)}
                        class="primary-button"
                    >
                        { "Clear" }
                    </button>
                </div>
            }
        } else {
            html! {}
        }
    }
}

async fn process_file(file: File) -> Result<AudioData, String> {
    // Read the file contents
    let array_buffer = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
        .await
        .map_err(|_| "Failed to read file".to_string())?;

    let array_buffer: ArrayBuffer = array_buffer.dyn_into().unwrap_throw();
    let uint8_array = Uint8Array::new(&array_buffer);
    let mut file_data = vec![0; uint8_array.length() as usize];
    uint8_array.copy_to(&mut file_data);

    // Send file to Tauri backend for processing
    let file_name = file.name();

    // Create a temporary file in the system temp directory
    let temp_path = format!("/tmp/{}", file_name);

    // Write the file to disk
    info!("Writing file to {}", temp_path);
    tauri::invoke::<Result<(), String>>(
        "plugin:fs|write_file",
        &serde_json::json!({
            "path": temp_path.clone(),
            "contents": file_data,
        }),
    )
    .await
    .map_err(|e| format!("Failed to save file: {}", e))?;
    info!("File written");

    // Load audio file in backend
    let _result = tauri::invoke::<Result<serde_json::Value, String>>(
        "analyze_audio_file",
        &serde_json::json!({
            "path": temp_path,
        }),
    )
    .await
    .map_err(|e| format!("Failed to analyze audio: {}", e))?;

    // Get samples and sample rate from backend
    let samples = tauri::invoke::<Result<Vec<f32>, String>>("get_samples", &serde_json::json!({}))
        .await
        .map_err(|e| format!("Failed to get samples: {}", e))?;

    let sample_rate =
        tauri::invoke::<Result<u32, String>>("get_sample_rate", &serde_json::json!({}))
            .await
            .map_err(|e| format!("Failed to get sample rate: {}", e))?;

    // Calculate duration
    let duration = samples.len() as f32 / sample_rate as f32;

    Ok(AudioData {
        samples,
        sample_rate,
        name: file_name,
        duration,
    })
}

async fn analyze_audio_data(data: &AudioData) -> Result<AnalysisResult, String> {
    // Get FFT analysis from backend
    let frequencies = tauri::invoke::<Result<Vec<serde_json::Value>, String>>(
        "compute_fft_command",
        &serde_json::json!({}),
    )
    .await
    .map_err(|e| format!("Failed to compute FFT: {}", e))?;

    // Convert frequencies to (freq, magnitude) pairs
    let frequencies: Vec<(f32, f32)> = frequencies
        .into_iter()
        .filter_map(|v| {
            let freq = v.get("frequency")?.as_f64()? as f32;
            let mag = v.get("magnitude")?.as_f64()? as f32;
            Some((freq, mag))
        })
        .collect();

    // Get spectrogram from backend
    let spectrogram = tauri::invoke::<Result<Vec<Vec<f32>>, String>>(
        "compute_spectrum_command",
        &serde_json::json!({}),
    )
    .await
    .map_err(|e| format!("Failed to compute spectrum: {}", e))?;

    // Get pitch estimation
    let pitch = tauri::invoke::<Result<Option<f32>, String>>(
        "estimate_pitch_command",
        &serde_json::json!({}),
    )
    .await
    .map_err(|e| format!("Failed to estimate pitch: {}", e))?;

    // Convert pitch to note if available
    let note = match pitch {
        Some(p) => Some(
            tauri::invoke::<Result<String, String>>(
                "frequency_to_note_command",
                &serde_json::json!({
                    "frequency": p
                }),
            )
            .await
            .unwrap_or_else(|_| "?".to_string()),
        ),
        None => None,
    };

    // Calculate RMS level
    let rms_level = data.samples.iter().map(|s| s * s).sum::<f32>() / data.samples.len() as f32;
    let rms_level = rms_level.sqrt();

    // Find peak frequency
    let peak_frequency = frequencies
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(freq, _)| *freq)
        .unwrap_or(0.0);

    Ok(AnalysisResult {
        frequencies,
        spectrogram,
        peak_frequency,
        pitch,
        note,
        rms_level,
    })
}
