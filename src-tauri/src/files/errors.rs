use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileError {

    #[error("Could not load file: {0}")]
    CouldNotLoadFile(anyhow::Error),

    #[error("Could not save file: {0}")]
    CouldNotSaveFile(anyhow::Error),

}