use std::fs;
use std::sync::{Arc, RwLock};
use home::home_dir;
use crate::files::errors::FileError;
use crate::Settings;

pub struct Storage {
    inner: Arc<RwLock<InnerStorage>>
}

pub struct InnerStorage {
    pub settings: Settings,
}

impl Storage {
    pub fn create(folder: &str) -> Result<Storage, FileError> {
        let home = home_dir().expect("Could not find home directory");
        let folder = home.join(folder);

        if let Ok(exists) = folder.try_exists() {
            if !exists {
                fs::create_dir(&folder).expect("Could not create folder");
            }
        } else {
            panic!("Could not check if folder exists");
        }

        let settings = Settings::new(&folder)?;

        Ok(Storage {
            inner: Arc::new(RwLock::new(InnerStorage {
                settings,
            })),
        })
    }

    pub fn extract(&self) -> Arc<RwLock<InnerStorage>> {
        self.inner.clone()
    }

}
