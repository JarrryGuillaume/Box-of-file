// src/commands/find_file.rs
use std::fs;
use std::path::Path;
use std::io;
use sha2::{Sha256, Digest};
use serde_json;
use crate::commands::global::get_global_bof_dir;

/// Find all directories where a file appears
pub fn find_file_directories(file_hash: &str) -> io::Result<Vec<String>> {
    let global_bof_dir = get_global_bof_dir()?;
    let inverse_table_path = global_bof_dir.join("inverse_table.json");

    if !inverse_table_path.exists() {
        return Ok(Vec::new());
    }

    let data = fs::read_to_string(&inverse_table_path)?;
    let inverse_table: serde_json::Value = serde_json::from_str(&data)?;

    let file_key = format!("sha256:{}", file_hash);
    if let Some(file_entry) = inverse_table["files"].get(&file_key) {
        let directories = file_entry["directories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        Ok(directories)
    } else {
        Ok(Vec::new())
    }
}

/// Compute the SHA-256 hash of a file
pub fn compute_file_hash(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}
