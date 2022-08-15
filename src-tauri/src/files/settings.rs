use std::path::{Path, PathBuf};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::files::errors::{FileError};
use std::fs;
use crate::Account;

const SETTINGS_FOLDER: &str = "settings";

pub struct Settings {
    pub accounts: ConfigurationFile<Vec<Account>>
}

pub struct ConfigurationFile<T: Serialize + DeserializeOwned> {
    path: PathBuf,
    pub contents: T,
}

impl<T: Serialize + DeserializeOwned> ConfigurationFile<T> {
    pub fn save(&self) -> Result<(), FileError> {

        if let Some(parents) = self.path.parent() {
            fs::create_dir_all(parents)
                .map_err(|err| FileError::CouldNotSaveFile(err.into()))?;
        }

        let contents = serde_json::to_vec(&self.contents)
            .map_err(|err| FileError::CouldNotSaveFile(err.into()))?;

        fs::write(&self.path, contents)
            .map_err(|err| FileError::CouldNotSaveFile(err.into()))?;

        Ok(())
    }
}

impl Settings {

    pub fn new(base: &PathBuf) -> Result<Self, FileError> {
        let base = base.join(SETTINGS_FOLDER);

        let accounts = Self::setup_file(&base, "accounts.json")?;

        Ok(Self {
            accounts,
        })
    }

    fn setup_file<T>(base: &PathBuf, path: impl AsRef<Path>) -> Result<ConfigurationFile<T>, FileError>
        where
            T: Serialize + DeserializeOwned + Default
    {
        let possible_file = Self::load_file(base, &path)?;

        let file = match possible_file {
            Some(file) => file,
            None => {
                let file = Self::create_file(base, path, T::default());
                file.save()?;
                file
            }
        };

        Ok(file)
    }

    fn create_file<T: Serialize + DeserializeOwned>(base: &PathBuf, path: impl AsRef<Path>, contents: T) -> ConfigurationFile<T> {
        ConfigurationFile {
            path: base.join(path),
            contents,
        }
    }

    fn load_file<T: Serialize + DeserializeOwned>(base: &PathBuf, path: impl AsRef<Path>) -> Result<Option<ConfigurationFile<T>>, FileError> {
        let path = base.join(path);

        let exists = path.try_exists()
            .map_err(|err| FileError::CouldNotLoadFile(err.into()))?;

        if !exists {
            return Ok(None);
        }

        let bytes = fs::read(&path)
            .map_err(|err| FileError::CouldNotLoadFile(err.into()))?;

        let contents = serde_json::from_slice(&bytes)
            .map_err(|err| FileError::CouldNotLoadFile(err.into()))?;

        Ok(Some(ConfigurationFile {
            path,
            contents,
        }))
    }

}