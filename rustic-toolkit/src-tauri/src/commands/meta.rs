use crate::types::GraphMetadata;

use rustic::meta::{get_filters, get_generators, get_sinks};

#[tauri::command]
pub fn get_graph_metadata() -> GraphMetadata {
    GraphMetadata {
        generators: get_generators(),
        filters: get_filters(),
        sinks: get_sinks(),
    }
}
