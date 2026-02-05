use log::info;

use crate::analysis::frequency_to_note;
use crate::error::AppError;
use crate::types::AudioSummary;

/// Convert a frequency (Hz) to the nearest musical note name.
#[tauri::command]
pub fn frequency_to_note_command(frequency: f32) -> Result<String, AppError> {
    Ok(frequency_to_note(frequency))
}

/// Serialize an AudioSummary to JSON and write it to disk.
#[tauri::command]
pub async fn save_analysis(path: String, summary: AudioSummary) -> Result<(), AppError> {
    info!("Saving analysis to: {}", path);

    let json = serde_json::to_string_pretty(&summary)
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    std::fs::write(&path, json)?;

    info!("Analysis saved successfully");
    Ok(())
}