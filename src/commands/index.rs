use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self};
use std::collections::HashMap;
use filetime::FileTime;
use uuid::Uuid;
use walkdir::WalkDir;
use sha2::{Sha256, Digest};
use serde_json::json;

use crate::commands::global::get_global_bof_dir;
use crate::data_struct::{FileMetadata, DirectoryMetadata};

pub fn canonicalize_path(path: &Path) -> io::Result<PathBuf> {
    let canonical_path = fs::canonicalize(path)?;
    let canonical_str = canonical_path.to_string_lossy().to_string();

    #[cfg(windows)]
    let cleaned_path = if canonical_str.starts_with(r"\\?\") {
        PathBuf::from(&canonical_str[4..])
    } else {
        canonical_path
    };

    #[cfg(not(windows))]
    let cleaned_path = canonical_path;

    Ok(cleaned_path)
}

fn update_inverse_table(file_path: &Path, file_name: &str, file_hash: &str) -> io::Result<()> {
    let global_bof_dir = get_global_bof_dir()?;
    let inverse_table_path = global_bof_dir.join("inverse_table.json");

    let mut inverse_table: serde_json::Value = if inverse_table_path.exists() {
        let data = fs::read_to_string(&inverse_table_path)?;
        serde_json::from_str(&data).unwrap_or(json!({ "files": {} }))
    } else {
        json!({ "files": {} })
    };

    let absolute_path = canonicalize_path(file_path)?;
    let parent_dir = absolute_path.parent().unwrap_or_else(|| Path::new("."));

    let file_key = format!("sha256:{}", file_hash);
    if let Some(file_entry) = inverse_table["files"].get_mut(&file_key) {
        let directories = file_entry["directories"].as_array_mut().unwrap();
        if !directories.contains(&serde_json::Value::String(parent_dir.to_string_lossy().to_string())) {
            directories.push(serde_json::Value::String(parent_dir.to_string_lossy().to_string()));
        }
    } else {
        inverse_table["files"][&file_key] = json!({
            "name": file_name,
            "directories": [parent_dir.to_string_lossy().to_string()]
        });
    }

    let inverse_table_json = serde_json::to_string_pretty(&inverse_table)?;
    fs::write(inverse_table_path, inverse_table_json)?;

    Ok(())
}

fn compute_file_hash(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

fn load_metadata(bof_dir: &Path) -> io::Result<(Vec<FileMetadata>, Vec<DirectoryMetadata>)> {
    let file_metadata_path = bof_dir.join("files.json");
    let dir_metadata_path = bof_dir.join("directories.json");

    let file_metadata = if file_metadata_path.exists() {
        let data = fs::read_to_string(&file_metadata_path)?;
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    };

    let dir_metadata = if dir_metadata_path.exists() {
        let data = fs::read_to_string(&dir_metadata_path)?;
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    };

    Ok((file_metadata, dir_metadata))
}

fn save_metadata(bof_dir: &Path, file_metadata: &[FileMetadata], dir_metadata: &[DirectoryMetadata]) -> io::Result<()> {
    let file_metadata_path = bof_dir.join("files.json");
    let dir_metadata_path = bof_dir.join("directories.json");

    let file_metadata_json = serde_json::to_string_pretty(file_metadata)?;
    let dir_metadata_json = serde_json::to_string_pretty(dir_metadata)?;

    fs::write(file_metadata_path, file_metadata_json)?;
    fs::write(dir_metadata_path, dir_metadata_json)?;

    Ok(())
}

pub fn collect_metadata(dir: &Path, bof_dir: &Path) -> io::Result<()> {
    let (existing_files, existing_dirs) = load_metadata(bof_dir)?;

    let mut file_map: HashMap<String, FileMetadata> = existing_files.into_iter().map(|f| (f.path.clone(), f)).collect();
    let mut dir_map: HashMap<String, DirectoryMetadata> = existing_dirs.into_iter().map(|d| (d.key.clone(), d)).collect();

    for entry in WalkDir::new(dir) {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Could not read entry: {}", err);
                continue;
            }
        };

        if entry.path().starts_with(bof_dir) {
            continue;
        }

        let metadata = match fs::symlink_metadata(entry.path()) {
            Ok(m) => m,
            Err(err) => {
                eprintln!("Could not get metadata for {:?}: {}", entry.path(), err);
                continue;
            }
        };

        let file_type = if metadata.is_dir() {
            "directory".to_string()
        } else if metadata.is_file() {
            "file".to_string()
        } else {
            "other".to_string()
        };

        let mtime = FileTime::from_last_modification_time(&metadata).unix_seconds();

        let ctime = FileTime::from_creation_time(&metadata)
            .map(|t| t.unix_seconds())
            .unwrap_or(0); 

        let size = metadata.len();

        let key = Uuid::new_v4().to_string();

        if metadata.is_dir() {
            let entries = fs::read_dir(entry.path())
                .unwrap()
                .map(|e| {
                    let e = e.unwrap();
                    let kind = if e.metadata().unwrap().is_dir() {
                        "directory".to_string()
                    } else {
                        "file".to_string()
                    };
                    (key.clone(), kind, e.file_name().to_string_lossy().to_string())
                })
                .collect();

            let dir_data = DirectoryMetadata {
                key: key.clone(),
                entries,
            };
            dir_map.insert(key, dir_data);
        } else {
            let file_hash = compute_file_hash(entry.path())?;
            let file_name = entry.file_name().to_string_lossy().to_string();

            update_inverse_table(bof_dir, &file_name, &file_hash)?;

            let file_data = FileMetadata {
                key,
                path: entry.path().to_string_lossy().to_string(),
                file_type,
                ctime: ctime as u64,
                mtime: mtime as u64,
                size,
            };
            file_map.insert(file_data.path.clone(), file_data);
        }
    }

    let updated_files: Vec<FileMetadata> = file_map.into_values().collect();
    let updated_dirs: Vec<DirectoryMetadata> = dir_map.into_values().collect();

    save_metadata(bof_dir, &updated_files, &updated_dirs)?;

    Ok(())
}