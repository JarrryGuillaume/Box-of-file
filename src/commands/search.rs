// src/commands/search.rs
use std::fs;
use std::io;
use std::io::empty;
use std::path::PathBuf;
use serde_json::{Value, json};

use crate::commands::global::get_global_bof_dir;

#[derive(Debug)]
pub struct SearchResult {
    pub file_name: String,
    pub directories: Vec<String>,
}

/// Search the global inverse table for files whose names contain `pattern`.
/// Returns a list of matching filenames plus the directories they appear in.
pub fn search_by_name(pattern: &str) -> io::Result<Vec<SearchResult>> {
    let global_bof_dir = get_global_bof_dir()?;
    let inverse_table_path = global_bof_dir.join("inverse_table.json");

    // If the inverse table doesn’t exist yet, just return empty
    if !inverse_table_path.exists() {
        return Ok(Vec::new());
    }

    // Load the inverse table JSON
    let data = fs::read_to_string(&inverse_table_path)?;
    let inverse_json: Value = serde_json::from_str(&data)
        .unwrap_or_else(|_| json!({ "files": {} }));

    let mut results = Vec::new();
    if let Some(files_obj) = inverse_json.get("files").and_then(|f| f.as_object()) {
        for (_file_key, file_info) in files_obj.iter() {
            // Safely extract the name from JSON
            let empty_array = Vec::new();
            let file_name = file_info.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();

            // Case-insensitive or plain substring match
            if file_name.to_lowercase().contains(&pattern.to_lowercase()) {
                // Extract all directories if any
                let dirs_array = file_info.get("directories")
                    .and_then(|d| d.as_array())
                    .unwrap_or(&empty_array);
                
                let mut directories = Vec::new();
                for d in dirs_array {
                    if let Some(dir_str) = d.as_str() {
                        directories.push(dir_str.to_string());
                    }
                }

                results.push(SearchResult {
                    file_name,
                    directories,
                });
            }
        }
    }

    Ok(results)
}
