use std::fs;
use std::path::{Path, PathBuf};
use std::io;

pub fn get_global_bof_dir() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;
    let global_bof_dir = home_dir.join("bof_global");

    if !global_bof_dir.exists() {
        fs::create_dir_all(&global_bof_dir)?;
    }

    Ok(global_bof_dir)
}