// src/commands/config_email.rs

use std::fs;
use std::io::{self, Write};
use serde::{Serialize, Deserialize};
use dirs::home_dir;
use crate::data_struct::EmailConfig;
use crate::commands::global::get_global_bof_dir;

/// Save the user’s email credentials and server info in a JSON config file.
/// Example path: ~/.bof_email_config.json
// src/commands/config_email.rs

pub fn config_email_command(
    address: &str,
    password: &str,
    server: &str,
    port_str: &str
) -> io::Result<()> {
    let port: u16 = port_str.parse().unwrap_or(587);

    let email_config = EmailConfig {
        address: address.to_string(),
        password: password.to_string(),
        server: server.to_string(),
        port,
    };

    let global_bof_dir = get_global_bof_dir()?;
    let config_path = global_bof_dir.join("email_config.json");

    let json_data = serde_json::to_string_pretty(&email_config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Serialize error: {e}")))?;

    let mut file = fs::File::create(&config_path)?;
    file.write_all(json_data.as_bytes())?;

    println!("Stored email config at {}", config_path.display());
    Ok(())
}
