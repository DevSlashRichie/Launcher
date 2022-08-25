use std::io;
use thiserror::Error;
use crate::auth_route::errors::AuthError;

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Could not download file: {0}")]
    DownloadError(#[from] reqwest::Error),
    #[error("Could not parse file: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Io Error: {0}")]
    IoError(#[from] io::Error),
    #[error("Asset not found")]
    NotFound,
    #[error("Authentication error: {0}")]
    AuthenticationError(#[from] AuthError),
}