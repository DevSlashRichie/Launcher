use std::fmt::Debug;
use std::path::{PathBuf};
use crate::version_manager::asset::VersionManifest;
use serde::{Deserialize, Serialize};

pub struct VersionConstruct {
    pub id: VersionId,
    pub at: PathBuf,
    pub manifest: VersionManifest,
    pub asset_path: PathBuf,
    pub libraries_path: PathBuf,
    pub natives_temp_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum VersionId {
    V1_19_2,
    V1_19_3,
}

impl ToString for VersionId {
    fn to_string(&self) -> String {
        match self {
            VersionId::V1_19_2 => "1.19.2".to_string(),
            VersionId::V1_19_3 => "1.19.3".to_string(),
        }
    }
}