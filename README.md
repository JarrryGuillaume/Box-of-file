# Box Of Files (bof)

**Box Of Files (bof)** is a Rust-based command-line application for indexing file metadata, searching for files by name or hash, and optionally emailing file metadata to recipients.

## Features

1. **Initialize** a local `.bof` directory in your current folder for storing metadata.  
2. **Index** all files in a directory (recursively), generating metadata (size, mtime, etc.) and updating a global inverse table in `~/bof_global`.  
3. **Find** specific files across all indexed folders by file hash.  
4. **Search** files by partial name (substring match).  
5. **Email** a file’s metadata using your configured SMTP credentials.  
6. **Clear** all `.bof` directories and the global inverse table.

## Table of Contents

- Installation
- Usage
  - bof init
  - bof index
  - bof findfile
  - bof search
  - bof config-email
  - bof send-email
  - bof clear-all
- How it Works
- Email Security Notes
- License

## Installation

1. **Clone** the repository:
```bash
   git clone https://github.com/yourusername/box_of_files.git  
   cd box_of_files
```

2. **Build** the binary:
```bash
   cargo build --release  
```
   This will produce a `bof` (or `bof.exe` on Windows) in `target/release`.

3. **(Optional) Install** the CLI to your PATH:
```bash
   cargo install --path .
```
   Now you can run `bof` from anywhere in your terminal.

## Usage

The basic command structure is:
bof <SUBCOMMAND> [OPTIONS...]

### bof init

Creates a local folder named `.bof` in the **current directory**. This folder stores `files.json` and `directories.json` metadata after you run `bof index`.

**Example**:
```bash
cd /path/to/my_project  
bof init
```

### bof index

Recursively indexes files under the **current directory**, writing metadata into `.bof/files.json`. Also updates a **global inverse table** (`inverse_table.json`) in `~/bof_global`.

**Example**:
```bash
cd /path/to/my_project  
bof index
```

### bof findfile

Find all directories that contain the given file (by computing its hash and looking it up in the global inverse table).

**Example**:
```bash
bof findfile --file /path/to/specific_file.pdf  
bof findfile --file myfile.txt
```

### bof search

Search for files by partial or substring matches of their names. Looks in the **global** inverse table for any file whose stored name matches the pattern.

**Example**:
bof search mypattern  
bof search .pdf

### bof config-email

Store your SMTP credentials in `~/bof_global/email_config.json`. This is used when sending email.

**Example**:
bof config-email --address "myaddress@outlook.com" --password "mypassword_or_app_password" --server "smtp.office365.com" --port "587"

- **address**: The email address you want to send from  
- **password**: The SMTP or app-specific password  
- **server**: SMTP server, e.g. `smtp.gmail.com` or `smtp.office365.com`  
- **port**: Typically `587` for STARTTLS

### bof send-email

Sends an email containing metadata about a specific file to the specified recipient.

**Example**:
bof send-email --file "/path/to/my_project/some_file.txt" --to "recipient@example.com"

- Looks up the local `.bof/files.json` for `some_file.txt`.  
- Loads your global email config from `~/bof_global/email_config.json`.  
- Sends an email with the file’s metadata.

### bof clear-all

Removes **all** `.bof` folders in every repo that has been indexed and cleans up the global inverse table in `~/bof_global/inverse_table.json`. Use with caution.

**Example**:
bof clear-all

## How it Works

1. **Local .bof Folder**  
   Each directory you `init` and `index` creates a `.bof` folder containing:  
   - `files.json` — storing metadata of each file (size, creation time, etc.)  
   - `directories.json` — storing entries for subdirectories.

2. **Global Repository**  
   A global folder `~/bof_global` stores:  
   - `inverse_table.json` — used to find which directories contain a given file (by hash).  
   - `email_config.json` — your SMTP configuration (username, password, server).

3. **File Lookup**  
   - `findfile` computes your file’s SHA-256 hash and queries `inverse_table.json`.  
   - `search` scans the “files” object in the global data for substring name matches.

4. **Email Sending**  
   - `config-email` writes your SMTP info to `~/bof_global/email_config.json`.  
   - `send-email` uses that config to open an SMTP connection, read a file’s metadata from `files.json`, and send a simple text email.

## Email Security Notes

- The **password** or **app password** for your email account is currently stored in **plaintext** in `~/bof_global/email_config.json`. If you’re concerned about security, limit access to that file or store credentials in a more secure manner.
- Gmail often requires an **App Password** if you use 2FA. Outlook might require an App Password or standard credentials with “Authenticated SMTP” enabled.
- All SMTP connections go over **STARTTLS** on port 587, so your credentials are encrypted during transmission.

## License

This project is free and open-source under the [MIT License](LICENSE). Contributions are welcome—feel free to open issues or submit PRs!
