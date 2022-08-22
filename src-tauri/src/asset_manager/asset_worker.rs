use std::path::PathBuf;
use std::{fs, io};
use std::io::{Read, Write};
use sha1::{Sha1, Digest};
use crate::asset_manager::asset::{Artifact, Version, VersionManifest, VersionsManifest};
use crate::asset_manager::errors::AssetError;
use crate::asset_manager::version::{VersionConstruct, VersionId};

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

    async fn get_manifest(&self, version: &VersionConstruct) -> Result<VersionManifest, AssetError> {
        let version_manifest = version.at.join("manifest.json");

        let manifest_bytes = match version_manifest.exists() {
            true => {
                let bytes = fs::read(version_manifest)?;
                Some(bytes)
            }

            false => {
                let versions = Self::fetch_version_list().await?;
                let version = versions.versions.into_iter().find(|v| v.id == version.id.to_string());

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

    async fn should_download_artifact(&self, path: &PathBuf, artifact: &Artifact) -> Result<bool, AssetError> {
        if !path.exists() {
            Ok(true)
        } else {
            let mut file = fs::File::open(&path)?;

            let sizes_match = file.metadata()?.len() == artifact.size;

            if !sizes_match {
                println!("{} has changed size", &artifact.url);
                return Ok(true);
            }

            let mut hasher = Sha1::new();

            let _ = io::copy(&mut file, &mut hasher)?;
            let bytes = hasher.finalize();
            let hash = format!("{:x}", bytes);

            if hash != artifact.sha1 {
                println!("{} has changed hash", &artifact.url);
                return Ok(true);
            }

            Ok(false)
        }
    }

    async fn download_file(&self, full_path: &PathBuf, url: &String) -> Result<(), AssetError> {
        let mut response = reqwest::get(url).await?;
        let mut file = fs::File::create(full_path)?;

        while let Some(response) = response.chunk().await? {
            file.write_all(&response)?;
        }

        Ok(())
    }

    async fn review_file(&self, at: &PathBuf, artifact: &Artifact) -> Result<(), AssetError> {
        let full_path = artifact.path(at);

        // In case the artifact owns a custom path, we need to make sure it exists
        if artifact.path.is_some() && !full_path.exists() {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?
            }
        }

        if self.should_download_artifact(&full_path, artifact).await? {
            self.download_file(&full_path, &artifact.url).await?;
        }

        Ok(())
    }

    pub async fn check_version(&self, version: VersionId) -> Result<(), AssetError> {
        let version = version.create(&self.folder);

        if !version.at.exists() {
            fs::create_dir(&version.at)?;
        }

        let manifest = self.get_manifest(&version).await?;

        self.review_file(&version.at, &manifest.downloads.client).await?;

        let lib_path = self.folder.join("libs");
        if !lib_path.exists() {
            fs::create_dir(&lib_path)?;
        }

        for lib in manifest.libraries {
            self.review_file(&lib_path, &lib.downloads.artifact).await?;
        }

        Ok(())
    }
}