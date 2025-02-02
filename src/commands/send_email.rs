use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use serde_json;

use crate::data_struct::{FileMetadata, EmailConfig};
use crate::commands::global::get_global_bof_dir;
use crate::commands::index::canonicalize_path; 
use std::process::Command;

pub fn send_file_metadata_email(file_path_str: &str, recipient: &str) -> io::Result<()> {
    let abs_file = canonicalize_path(Path::new(file_path_str))?;

    let bof_dir = find_bof_dir_for_path(&abs_file).ok_or_else(|| {
        io::Error::new(
            ErrorKind::NotFound,
            format!("No .bof folder found for file: {}", abs_file.display()),
        )
    })?;

    let files_json = bof_dir.join("files.json");
    if !files_json.exists() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!("{} not found; did you run 'bof index'?", files_json.display()),
        ));
    }

    let data = fs::read_to_string(&files_json)?;
    let all_files: Vec<FileMetadata> = serde_json::from_str(&data)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, format!("JSON parse error: {e}")))?;

    let maybe_meta = all_files.iter().find(|m| {
        let file_in_index = bof_dir
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(&m.path);

        if let Ok(index_abs) = canonicalize_path(&file_in_index) {
            index_abs == abs_file
        } else {
            false
        }
    });

    let file_meta = match maybe_meta {
        Some(m) => m,
        None => {
            return Err(io::Error::new(
                ErrorKind::NotFound,
                format!(
                    "File '{}' not found in {}",
                    abs_file.display(),
                    files_json.display()
                ),
            ));
        }
    };

    let email_config = load_email_config()?;

    let body = format!(
        "Hello,\n\nHere is the metadata for the file you requested:\n\n\
         Path: {}\n\
         Type: {}\n\
         Size: {} bytes\n\
         Created: {}\n\
         Modified: {}\n\n\
         Sent from Rust.\n\
         Enjoy!\n",
        file_meta.path,
        file_meta.file_type,
        file_meta.size,
        file_meta.ctime,
        file_meta.mtime
    );

    send_email_via_python(
        &email_config.address,   // outlook_username
        &email_config.password,  // outlook_password
        &email_config.address,   // from_address
        recipient,               // to_address
        &body,
        &email_config.server,
    )?;

    Ok(())
}

fn find_bof_dir_for_path(file_path: &Path) -> Option<PathBuf> {
    let mut current = file_path.parent();

    while let Some(dir) = current {
        let candidate = dir.join(".bof");
        if candidate.is_dir() {
            return Some(candidate);
        }
        current = dir.parent();
    }
    None
}

fn load_email_config() -> io::Result<EmailConfig> {
    let global_bof_dir = get_global_bof_dir()?;
    let config_path = global_bof_dir.join("email_config.json");

    if !config_path.exists() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!("Global email config not found at {}", config_path.display()),
        ));
    }

    let data = fs::read_to_string(&config_path)?;
    let cfg: EmailConfig = serde_json::from_str(&data).map_err(|e| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Could not parse config: {e}"),
        )
    })?;

    Ok(cfg)
}

fn get_python_script_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR"); 
    
    let script_path = Path::new(manifest_dir)
        .join("src")
        .join("python")
        .join("send_email.py");
    
    script_path
}

pub fn send_email_via_python(
    outlook_username: &str,
    outlook_password: &str,
    from_address: &str,
    to_address: &str,
    body: &str,
    smtp_server: &str,
) -> io::Result<()> {
    let python_script_path = get_python_script_path(); 

    let status = Command::new("python")
        .arg(python_script_path)
        .arg(outlook_username)
        .arg(outlook_password)
        .arg(from_address)
        .arg(to_address)
        .arg(smtp_server)
        .arg(body) 
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("Python send_email script failed with exit code: {}", status),
        ));
    }

    println!("Email sent successfully via Python child process!");
    Ok(())
}
