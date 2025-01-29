// src/commands/clear.rs
use std::fs;
use std::io;
use std::path::PathBuf;
use serde_json;
use crate::commands::global::get_global_bof_dir;

/// Remove all `.bof` directories listed in the inverse table
pub fn clear_all_bof_dirs() -> io::Result<()> {
    let global_bof_dir = get_global_bof_dir()?;
    let inverse_table_path = global_bof_dir.join("inverse_table.json");

    // Check if the inverse table exists
    if !inverse_table_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The inverse table does not exist. Run 'bof index' to create it.",
        ));
    }

    // Load the inverse table
    let data = fs::read_to_string(&inverse_table_path)?;
    let inverse_table: serde_json::Value = serde_json::from_str(&data)?;

    // Iterate through all files in the inverse table
    if let Some(files) = inverse_table.get("files") {
        for (_, file_entry) in files.as_object().unwrap() {
            if let Some(directories) = file_entry.get("directories") {
                for dir in directories.as_array().unwrap() {
                    let dir_path = dir.as_str().unwrap();
                    let bof_dir = PathBuf::from(dir_path).join(".bof");

                    // Remove the `.bof` directory if it exists
                    if bof_dir.exists() {
                        fs::remove_dir_all(&bof_dir)?;
                        println!("Removed: {:?}", bof_dir);
                    }
                }
            }
        }
    }

    println!("All .bof directories removed.");
    Ok(())
}