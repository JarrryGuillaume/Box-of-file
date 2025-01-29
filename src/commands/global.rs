// src/commands/global.rs
use std::fs;
use std::path::{Path, PathBuf};
use std::io;

/// Get the path to the global BOF directory (e.g., C:/Users/<your-username>/bof_global)
pub fn get_global_bof_dir() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;
    let global_bof_dir = home_dir.join("bof_global");

    // Create the global directory if it doesn't exist
    if !global_bof_dir.exists() {
        fs::create_dir_all(&global_bof_dir)?;
    }

    Ok(global_bof_dir)
}