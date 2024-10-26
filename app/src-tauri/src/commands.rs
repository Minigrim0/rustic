use rustic::{filter_metadata, filters::FilterMetadata};

#[tauri::command]
pub fn get_filters() -> Vec<FilterMetadata> {
    filter_metadata()
}
