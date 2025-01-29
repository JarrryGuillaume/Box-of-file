// src/dataStruct.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMetadata {
    pub key: String,
    pub path: String,
    pub file_type: String,
    pub ctime: u64,
    pub mtime: u64,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectoryMetadata {
    pub key: String,
    pub entries: Vec<(String, String, String)>, // (KEY, KIND, NAME)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailConfig {
    pub address: String,
    pub password: String,
    pub server: String,
    pub port: u16,
}