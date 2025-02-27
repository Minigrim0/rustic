/// The FS module is used to interact with the filesystem.
/// Its purpose is to help organize the filesystem and provide a way to interact with it.

use std::path::{Path, PathBuf};
use std::fs;
use log::error;

fn check_out_dir() -> Result<PathBuf, ()> {
    let out_dir = Path::new("./.dist/");
    if !out_dir.exists() {
        if let Err(e) = fs::create_dir(out_dir) {
            error!("Failed to create output directory: {}", e);
            return Err(());
        }
    }
    Ok(out_dir.to_path_buf())
}

fn build_full_path(path: &Path) -> Result<(), ()> {
    if !path.exists() {
        if let Err(e) = fs::create_dir(path) {
            error!("Failed to create output directory: {}", e);
            return Err(());
        }
    }
    Ok(())
}

pub fn build_path(module: &str, filename: &str) -> Result<PathBuf, ()> {
    let base_path = check_out_dir()?;
    let full_path = base_path.join(module);
    build_full_path(&full_path)?;
    Ok(full_path.join(filename))
}

pub fn build_timed_path(module: &str, filename: &str) -> Result<PathBuf, ()> {
    let base_path = check_out_dir()?;
    let full_path = base_path.join(module);
    let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let full_path = full_path.join(format!("{}_{}", timestamp, filename));
    Ok(full_path)
}
