use std::path::{PathBuf};
use crate::version_manager::asset::VersionManifest;

pub struct VersionConstruct {
    pub id: VersionId,
    pub at: PathBuf,
    pub manifest: VersionManifest,
    pub asset_path: PathBuf,
    pub libraries_path: PathBuf,
    pub natives_temp_path: PathBuf,
}

pub enum VersionId {
    V1_19_2,
}

impl ToString for VersionId {
    fn to_string(&self) -> String {
        match self {
            VersionId::V1_19_2 => "1.19.2".to_string(),
        }
    }
}