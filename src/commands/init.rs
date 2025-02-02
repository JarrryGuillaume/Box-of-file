use std::fs;
use std::path::{Path, PathBuf};
use std::io;

pub fn init_bof_directory(root: &Path) -> io::Result<PathBuf> {
    let bof_dir = root.join(".bof");
    if !bof_dir.exists() {
        fs::create_dir_all(&bof_dir)?;
        println!("Created .bof directory at: {:?}", bof_dir);
    }
    Ok(bof_dir)
}