// src/commands/send_email.rs

use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use serde_json;

use crate::data_struct::{FileMetadata, EmailConfig};
use crate::commands::global::get_global_bof_dir;
use crate::commands::index::canonicalize_path; // or define your own
use std::process::Command;

///
/// Send an email with metadata for `file_path_str`.
/// - Finds the local .bof/files.json to get file metadata.
/// - Loads the global config (~/bof_global/email_config.json).
/// - Spawns a Python script to do the actual SMTP send.
///
pub fn send_file_metadata_email(file_path_str: &str, recipient: &str) -> io::Result<()> {
    // 1) Canonicalize the input path so we have an absolute path
    let abs_file = canonicalize_path(Path::new(file_path_str))?;

    // 2) Find the .bof folder upward from that file
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

    // 3) Read .bof/files.json and locate the matching FileMetadata
    let data = fs::read_to_string(&files_json)?;
    let all_files: Vec<FileMetadata> = serde_json::from_str(&data)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, format!("JSON parse error: {e}")))?;

    let maybe_meta = all_files.iter().find(|m| {
        // m.path might be relative; combine it with the .bof's parent
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

    // 4) Load global email config (~/bof_global/email_config.json)
    let email_config = load_email_config()?;

    // 5) Build the body
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

    // 6) Call our Python-based function
    // We pass email_config.address as both the username *and* from_address
    // if that's how you log in. Or if Outlook requires a separate "username"
    // field (some do), you can store that in EmailConfig or use email_config.address.
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

/// Climbs upward from `file_path` until it finds a directory containing `.bof`.
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

/// Load config from `~/bof_global/email_config.json`
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
    // This is the path to your Cargo.toml directory
    let manifest_dir = env!("CARGO_MANIFEST_DIR"); 
    
    // Construct the absolute path to `src/python/send_email.py`
    let script_path = Path::new(manifest_dir)
        .join("src")
        .join("python")
        .join("send_email.py");
    
    script_path
}

/// Actually spawns Python, passing it the arguments needed by your `outlook_test.py`.
pub fn send_email_via_python(
    outlook_username: &str,
    outlook_password: &str,
    from_address: &str,
    to_address: &str,
    body: &str,
    smtp_server: &str,
) -> io::Result<()> {
    // We assume `outlook_test.py` is in the same folder or specify an absolute path.
    // For example: "C:/Users/guill/Documents/box_of_files/bof/outlook_test.py"

    // Our python script is in "python\send_email.py" from the project root
    let python_script_path = get_python_script_path(); 

    // Check if body is multiline. If it has newlines, passing as a single arg might be tricky
    // but let's keep it simple. We'll pass it as one argument.
    // The Python code reads `sys.argv[6]` as the body.
    let status = Command::new("python")
        .arg(python_script_path)
        .arg(outlook_username)
        .arg(outlook_password)
        .arg(from_address)
        .arg(to_address)
        .arg(smtp_server)
        .arg(body)  // pass the entire body as the 7th argument
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
