use std::path::PathBuf;
use std::fs;
use crate::asset_manager::asset::{Version, VersionManifest, VersionsManifest};
use crate::asset_manager::errors::AssetError;

const VERSION_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

pub struct AssetManager {
    folder: PathBuf,
}

impl AssetManager {
    pub fn new(base: &PathBuf) -> AssetManager {
        let base = base.join("assets");

        if !base.exists() {
            fs::create_dir(&base)
                .expect("Could not create assets folder");
        }

        AssetManager {
            folder: base
        }
    }

    async fn fetch_version_list() -> Result<VersionsManifest, AssetError> {
        let res = reqwest::get(VERSION_MANIFEST).await?
            .json::<VersionsManifest>()
            .await?;

        Ok(res.into())
    }

    async fn fetch_version(version: Version) -> Result<Vec<u8>, AssetError> {
        let res = reqwest::get(version.url)
            .await?
            .bytes()
            .await?;

        Ok(res.into())
    }

    pub async fn get_version(&self, version: &str) -> Result<VersionManifest, AssetError> {
        let version_folder = self.folder.join(version);
        let version_manifest = version_folder.join("manifest.json");

        let manifest_bytes = match version_manifest.exists() {
            true => {
                let bytes = fs::read(version_manifest)?;
                Some(bytes)
            }

            false => {

                if !version_folder.exists() {
                    fs::create_dir(&version_folder)?;
                }

                let versions = Self::fetch_version_list().await?;
                let version = versions.versions.into_iter().find(|v| v.id == version);

                if let Some(version) = version {
                    let version = Self::fetch_version(version).await?;

                    fs::write(version_manifest, &version)?;

                    Some(version)
                } else { None }
            }
        };

        let manifest = manifest_bytes
            .map_or(Ok(None), |manifest| {
                serde_json::from_slice::<VersionManifest>(&manifest)
                    .map(Some)
            })?;

        match manifest {
            None => {
                Err(AssetError::NotFound)
            }

            Some(manifest) => Ok(manifest)
        }
    }
}
