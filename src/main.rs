mod data_struct;
mod commands;

use clap::{Command, Arg};
use std::path::Path;
use commands::{init, index, find_file, clear, search, email_config, send_email};

fn main() {
    let matches = Command::new("bof")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("A tool to index and manage file metadata")
        .subcommand(
            Command::new("init")
                .about("Initialize a .bof directory in the current folder"),
        )
        .subcommand(
            Command::new("index")
                .about("Index files and directories in the current folder"),
        )
        .subcommand(
            Command::new("findfile")
                .about("Find all directories where a file appears")
                .arg(
                    Arg::new("file")
                        .help("The file to search for")
                        .required(true)
                        // Replacing `takes_value(true)` with:
                        .num_args(1),
                ),
        )
        .subcommand(
            Command::new("clear-all")
                .about("Remove all .bof directories listed in the inverse table"),
        )
        .subcommand(
            Command::new("search") 
                .about("Search for files by partial name")
                .arg(Arg::new("pattern")
                    .help("The substring to match against filenames")
                    .required(true)
                    .index(1))
        )
        .subcommand(
            Command::new("config-email")
                .about("Save SMTP config (email, password, server, etc.)")
                .arg(
                    Arg::new("address")
                        .long("address")
                        .help("Your email address (sender)")
                        .required(true)
                        .num_args(1), // Clap 4 style
                )
                .arg(
                    Arg::new("password")
                        .long("password")
                        .help("Password or app-specific password")
                        .required(true)
                        .num_args(1), 
                )
                .arg(
                    Arg::new("server")
                        .long("server")
                        .help("SMTP server, e.g. smtp.gmail.com")
                        // not strictly required, but we set default
                        .default_value("smtp.gmail.com")
                        .num_args(1),
                )
                .arg(
                    Arg::new("port")
                        .long("port")
                        .help("SMTP port, e.g. 587 for STARTTLS")
                        .default_value("587")
                        .num_args(1),
                ),
        )
        // NEW: send-email
        .subcommand(
            Command::new("send-email")
                .about("Send an email with the metadata of a specified file")
                .arg(
                    Arg::new("file")
                        .long("file")
                        .help("Which file to fetch metadata for")
                        .required(true)
                        .num_args(1),
                )
                .arg(
                    Arg::new("to")
                        .long("to")
                        .help("Recipient's email address")
                        .required(true)
                        .num_args(1),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            let current_dir = Path::new(".");
            if let Err(e) = init::init_bof_directory(current_dir) {
                eprintln!("Failed to initialize .bof directory: {}", e);
            }
        }
        Some(("index", _)) => {
            let current_dir = Path::new(".");
            let bof_dir = current_dir.join(".bof");
            if let Err(e) = index::collect_metadata(current_dir, &bof_dir) {
                eprintln!("Failed to index files: {}", e);
            }
        }
        Some(("findfile", sub_matches)) => {
            let file_path = sub_matches.get_one::<String>("file").unwrap();

            // Compute the file's hash
            let file_hash = match find_file::compute_file_hash(Path::new(file_path)) {
                Ok(hash) => hash,
                Err(e) => {
                    eprintln!("Failed to compute file hash: {}", e);
                    return;
                }
            };

            // Find directories where the file appears
            match find_file::find_file_directories(&file_hash) {
                Ok(directories) => {
                    if directories.is_empty() {
                        println!("File not found in any indexed directory.");
                    } else {
                        println!("File found in the following directories:");
                        for dir in directories {
                            println!("- {}", dir);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to query inverse table: {}", e);
                }
            }
        }
        Some(("clear-all", _)) => {
            if let Err(e) = clear::clear_all_bof_dirs() {
                eprintln!("{}", e);
            }
        }
        Some(("search", sub_matches)) => {
            let pattern = sub_matches.get_one::<String>("pattern").unwrap();
            match search::search_by_name(pattern) {
                Ok(matches) => {
                    if matches.is_empty() {
                        println!("No files match '{}'.", pattern);
                    } else {
                        println!("Found {} matches for '{}':\\n", matches.len(), pattern);
                        for entry in matches {
                            println!("File name: {}", entry.file_name);
                            println!("Directories:");
                            for dir in entry.directories {
                                println!("  - {}", dir);
                            }
                            println!(""); // blank line
                        }
                    }
                }
                Err(e) => eprintln!("Failed to search file: {}", e),
            }
        }
        Some(("config-email", sub_matches)) => {
            let address = sub_matches.get_one::<String>("address").unwrap();
            let password = sub_matches.get_one::<String>("password").unwrap();
            let server = sub_matches.get_one::<String>("server").unwrap();
            let port = sub_matches.get_one::<String>("port").unwrap();

            match email_config::config_email_command(address, password, server, port) {
                Ok(_) => println!("Email config saved successfully!"),
                Err(e) => eprintln!("Failed to save email config: {}", e),
            }
        }
        Some(("send-email", sub_matches)) => {
            let file_path = sub_matches.get_one::<String>("file").unwrap();
            let recipient = sub_matches.get_one::<String>("to").unwrap();

            match send_email::send_file_metadata_email(file_path, recipient) {
                Ok(_) => println!("Email sent!"),
                Err(e) => eprintln!("Failed to send email: {}", e),
            }
        }
        _ => {
            println!("No valid command provided. Use one of:");
            println!("  bof init");
            println!("  bof index");
            println!("  bof findfile --file <FILE>");
            println!("  bof clear-all");
            println!("  bof config-email --address <ADDRESS> --password <PASSWORD> --server <SMTP> --port <PORT>");
            println!("  bof send-email --file <FILE> --to <RECIPIENT>");
        }
    }
}
