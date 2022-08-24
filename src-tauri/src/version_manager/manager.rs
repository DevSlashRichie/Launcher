use std::path::PathBuf;
use std::{fs, io};
use std::io::Write;
use regex::Regex;
use sha1::{Sha1, Digest};
use tauri::api::process::Command;
use crate::Account;
use crate::version_manager::asset::{Argument, ArgumentValue, Artifact, Version, VersionManifest, VersionsManifest};
use crate::version_manager::errors::AssetError;
use crate::version_manager::version::{VersionConstruct, VersionId};

const VERSION_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

// We can add clone because at the end is only a PathBuf
#[derive(Clone)]
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

    async fn get_manifest(&self, at: &PathBuf, version: &VersionId) -> Result<VersionManifest, AssetError> {
        let version_manifest = at.join("manifest.json");

        let manifest_bytes = match version_manifest.exists() {
            true => {
                let bytes = fs::read(version_manifest)?;
                Some(bytes)
            }

            false => {
                let versions = Self::fetch_version_list().await?;
                let version = versions.versions.into_iter().find(|v| v.id == version.to_string());

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
        let full_path = artifact.derive_path(at);

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

    async fn check_version(&self, version: &VersionConstruct) -> Result<(), AssetError> {
        let manifest = &version.manifest;

        // We check if the client is present
        self.review_file(&version.at, &manifest.downloads.client).await?;

        let lib_path = &version.libraries_path;
        if !lib_path.exists() {
            fs::create_dir(&lib_path)?;
        }

        if !version.natives_temp_path.exists() {
            fs::create_dir(&version.natives_temp_path)?;
        }

        // We check if the libs are present
        for lib in &manifest.libraries {
            if let Some(rules) = &lib.rules {
                let invalid = rules.iter()
                    .any(|rule| !rule.is_valid());

                if invalid { continue; }
            }

            self.review_file(&lib_path, &lib.downloads.artifact).await?;
        }

        let asset_index_path = &version.asset_path.join("indexes");

        if !asset_index_path.exists() {
            fs::create_dir_all(&asset_index_path)?;
        }

        // We check if the assets are present
        self.review_file(&asset_index_path, &manifest.asset_index).await?;

        Ok(())
    }

    pub fn build_arguments(
        &self,
        version: &VersionConstruct,
        minecraft_account: Account,
    ) -> Vec<String> {
        let regex_to_replace = Regex::new(r"\$\{([\S_]+)}").unwrap();

        let class_path = version.manifest.libraries.iter()
            .filter(|lib| match &lib.rules {
                None => true,
                Some(rules) => rules.iter()
                    .all(|x| x.is_valid()),
            })
            .filter_map(|lib| {
                match &lib.downloads.artifact.path {
                    Some(path) => Some(version.libraries_path.join(path).to_str().unwrap().to_string()),
                    None => None
                }
            })
            .map(|lib| lib + ":")
            .collect::<String>();

        let client_path = version.at.join(version.manifest.downloads.client.file_name());
        let class_path = class_path + client_path.to_str().unwrap();

        // First we set the jvm parameters
        let arguments = Vec::new()
            .iter()
            .chain(&version.manifest.arguments.jvm)
            .chain(&vec![Argument::Plain(version.manifest.main_class.clone())])
            .chain(&version.manifest.arguments.game)
            .flat_map(|arg|
                match arg {
                    Argument::Plain(value) => vec![value.clone()],
                    Argument::WithRules { value, rules } => {
                        if rules.iter().any(|rule| !rule.is_valid()) {
                            vec![]
                        } else {
                            match value {
                                ArgumentValue::String(arg) => vec![arg.clone()],
                                ArgumentValue::Array(args) => args.clone(),
                            }
                        }
                    }
                }
            )
            .map(|arg| {
                let group = regex_to_replace.captures(&arg);

                if let Some(group) = group {
                    let key = group.get(1).unwrap();
                    let replacement = match key.as_str() {
                        "auth_player_name" => minecraft_account.profile.name.clone(),
                        "version_name" => version.id.to_string(),
                        "game_directory" => version.at.to_str().unwrap().to_string(),
                        "assets_root" => version.asset_path.to_str().unwrap().to_string(),
                        "assets_index_name" => version.manifest.asset_index.id.as_ref().unwrap().clone(),
                        "auth_uuid" => minecraft_account.profile.id.clone(),
                        "auth_access_token" => minecraft_account.mc.access_token.clone(),
                        "clientid" => "client_id_random".to_string(),
                        "auth_xuid" => "auth_xuid_random".to_string(),
                        "user_type" => "mojang".to_string(),
                        "version_type" => "release".to_string(),
                        "natives_directory" => version.natives_temp_path.to_str().unwrap().to_string(),
                        "launcher_name" => "Cognatize".to_string(),
                        "launcher_version" => "1.0.0".to_string(),
                        "classpath" => class_path.clone(),
                        _ => "".to_string()
                    };

                    regex_to_replace.replace(&arg, &replacement).to_string()
                } else {
                    arg
                }
            })
            .collect::<Vec<_>>();

        arguments
    }

    pub async fn load_version(&self, version: VersionId, minecraft_account: Account) -> Result<(), AssetError> {
        let at = self.folder.join(&version.to_string());

        if !at.exists() {
            fs::create_dir(&at)?;
        }

        let manifest = self.get_manifest(&at, &version).await?;

        let asset_path = at.join("assets");
        let libraries_path = at.join("libs");
        let natives_temp_path = at.join("natives");

        let construct = VersionConstruct {
            id: version,
            at,
            manifest,
            asset_path,
            libraries_path,
            natives_temp_path,
        };

        self.check_version(&construct).await?;

        let args = self.build_arguments(
            &construct,
            minecraft_account,
        );

        let handle = tokio::spawn(async move {
            let (mut rx, _) = Command::new("java")
                .args(args)
                .current_dir(construct.at.clone())
                .spawn()
                .expect("Failed to start java");

            while let Some(event) = rx.recv().await {
                println!("MINECRAFT: {:?}", event);
            }
        });

        let _ = tokio::join!(handle);

        Ok(())
    }
}