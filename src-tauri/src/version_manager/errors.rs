use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Could not download file: {0}")]
    DownloadError(#[from] reqwest::Error),
    #[error("Could not parse file: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Io Error: {0}")]
    IoError(#[from] io::Error),
    #[error("Asset not found")]
    NotFound,
}