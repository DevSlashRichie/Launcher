use std::path::PathBuf;
use std::{fs, io};
use std::io::Write;
use futures::StreamExt;
use regex::Regex;
use serde_json::Value;
use sha1::{Sha1, Digest};
use tauri::api::process::{Command, CommandEvent};
use crate::{Account, auth_route, Game};
use crate::version_manager::asset::{Argument, ArgumentValue, Artifact, Version, VersionManifest, VersionsManifest};
use crate::version_manager::errors::ManagerError;
use crate::version_manager::version::{VersionConstruct, VersionId};

use tracing::{info, error, warn, span, instrument, Instrument};

const VERSION_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
const MINECRAFT_RESOURCES: &str = "https://resources.download.minecraft.net/";

// We can add clone because at the end is only a PathBuf
#[derive(Clone, Debug)]
pub struct AssetManager {
    versions: PathBuf,
    instances: PathBuf,
    common: PathBuf,
}

impl AssetManager {
    pub fn new(base: &PathBuf) -> AssetManager {
        AssetManager {
            versions: base.join("versions"),
            instances: base.join("instances"),
            common: base.join("common"),
        }
    }

    pub fn ensure_exists(&self) -> io::Result<()> {
        fs::create_dir_all(&self.versions)?;
        fs::create_dir_all(&self.instances)?;
        fs::create_dir_all(&self.common)?;
        Ok(())
    }

    async fn fetch_version_list() -> Result<VersionsManifest, ManagerError> {
        let res = reqwest::get(VERSION_MANIFEST).await?
            .json::<VersionsManifest>()
            .await?;

        Ok(res.into())
    }

    async fn fetch_version(version: Version) -> Result<Vec<u8>, ManagerError> {
        let res = reqwest::get(version.url)
            .await?
            .bytes()
            .await?;

        Ok(res.into())
    }

    async fn get_manifest(&self, at: &PathBuf, version: &VersionId) -> Result<VersionManifest, ManagerError> {
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
                Err(ManagerError::NotFound)
            }

            Some(manifest) => Ok(manifest)
        }
    }

    async fn should_download_artifact(&self, path: &PathBuf, artifact: &Artifact) -> Result<bool, ManagerError> {
        if !path.exists() {
            Ok(true)
        } else {
            let mut file = fs::File::open(&path)?;

            let sizes_match = file.metadata()?.len() == artifact.size;

            if !sizes_match {
                warn!("Changed size: {}", &artifact.file_name());
                return Ok(true);
            }

            let mut hasher = Sha1::new();

            let _ = io::copy(&mut file, &mut hasher)?;
            let bytes = hasher.finalize();
            let hash = format!("{:x}", bytes);

            if hash != artifact.sha1 {
                warn!("Changed Hash: {}", &artifact.file_name());
                return Ok(true);
            }

            Ok(false)
        }
    }

    async fn download_file(&self, full_path: &PathBuf, url: &String) -> Result<(), ManagerError> {
        let mut response = reqwest::get(url).await?;
        let mut file = fs::File::create(full_path)?;

        while let Some(response) = response.chunk().await? {
            file.write_all(&response)?;
        }

        Ok(())
    }

    async fn review_file(&self, at: &PathBuf, artifact: &Artifact) -> Result<(), ManagerError> {
        let full_path = artifact.derive_path(at);

        // In case the artifact owns a custom path, we need to make sure it exists
        if artifact.path.is_some() && !full_path.exists() {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?
            }
        }

        if self.should_download_artifact(&full_path, artifact).await? {
            info!("Downloading: {}", artifact.id());
            self.download_file(&full_path, &artifact.url).await?;
        }

        Ok(())
    }

    async fn check_version(&self, version: &VersionConstruct) -> Result<(), ManagerError> {
        let manifest = &version.manifest;

        // We check if the client is present
        info!("Performing client check");
        self.review_file(&version.at, &manifest.downloads.client).await?;

        let lib_path = &version.libraries_path;
        if !lib_path.exists() {
            fs::create_dir(&lib_path)?;
        }

        if !version.natives_temp_path.exists() {
            fs::create_dir(&version.natives_temp_path)?;
        }

        // We check if the libs are present
        info!("Performing libraries check.");
        let libraries = manifest.libraries.iter()
            .filter(|lib|
                lib.rules.is_none() || lib.rules
                    .as_ref()
                    .unwrap()
                    .iter()
                    .all(|rule| rule.is_valid())
            )
            .map(|lib| lib.downloads.artifact.clone())
            .collect::<Vec<_>>();

        self.review_list_of_artifacts(lib_path.clone(), libraries).await?;

        let asset_index_path = &version.asset_path.join("indexes");

        if !asset_index_path.exists() {
            fs::create_dir_all(&asset_index_path)?;
        }

        // We check if the assets are present

        info!("Performing asset index check.");
        self.review_file(&asset_index_path, &manifest.asset_index).await?;

        let objects_path = version.asset_path.join("objects");

        if !objects_path.exists() {
            fs::create_dir(&objects_path)?;
        }

        info!("Converting assets to artifact format.");
        let objects = self.get_object_artifacts(version)?;

        info!("Performing asset objects check.");
        self.review_list_of_artifacts(objects_path, objects).await?;

        Ok(())
    }

    async fn review_list_of_artifacts(&self, at: PathBuf, artifacts: Vec<Artifact>) -> Result<(), ManagerError> {
        let buffer_size = num_cpus::get();
        futures::stream::iter(artifacts.into_iter()
            .map(|chunk| {
                let manager = self.clone();
                let path = at.clone();
                tokio::spawn(async move {
                    if let Err(err) = manager.review_file(&path, &chunk).await {
                        error!("Artifact error check: {}", err);
                    }
                    chunk.id()
                })
            }))
            .buffer_unordered(buffer_size)
            .inspect(|name| {
                match name {
                    Ok(name) => info!("Artifact loaded: {}", name),
                    Err(err) => error!("Error while reviewing artifact: {}", err),
                };
            })
            .collect::<Vec<_>>()
            .await;

        Ok(())
    }

    pub fn build_arguments(
        &self,
        version: VersionConstruct,
        minecraft_account: Account,
        game: Game,
        instance_path: &PathBuf,
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
                        "version_name" => game.version.to_string(),
                        "game_directory" => instance_path.to_str().unwrap().to_string(),
                        "assets_root" => version.asset_path.to_str().unwrap().to_string(),
                        "assets_index_name" => version.manifest.asset_index.id.as_ref().unwrap().clone(),
                        "auth_uuid" => minecraft_account.profile.id.clone(),
                        "auth_access_token" => minecraft_account.mc.access_token.clone(),
                        // "clientid" => "client_id_random".to_string(),
                        // "auth_xuid" => "auth_xuid_random".to_string(),
                        "user_type" => "mojang".to_string(),
                        "version_type" => "release".to_string(),
                        "natives_directory" => version.natives_temp_path.to_str().unwrap().to_string(),
                        "launcher_name" => "Cognatize".to_string(),
                        "launcher_version" => "1.0.0".to_string(),
                        "classpath" => class_path.clone(),
                        _ => key.as_str().to_string()
                    };

                    regex_to_replace.replace(&arg, &replacement).to_string()
                } else {
                    arg
                }
            })
            .collect::<Vec<_>>();

        println!("{:?}", arguments.join(" "));

        arguments
    }

    fn get_object_artifacts(&self, version: &VersionConstruct) -> Result<Vec<Artifact>, ManagerError> {
        let file_name = version.manifest.asset_index.file_name();
        let index_file = version.asset_path.join("indexes").join(file_name);
        let load_file = fs::read(&index_file)?;
        let file = serde_json::from_slice::<Value>(&load_file)?;

        let objects = file["objects"].as_object().unwrap();

        // convert the objects to a list of artifacts
        let artifacts = objects.iter()
            .map(|(key, value)| {
                let hash = value.get("hash").unwrap().as_str().unwrap();
                let sub = hash.chars().take(2).collect::<String>();
                let size = value.get("size").unwrap().as_u64().unwrap();

                let path = sub + "/" + hash;
                let url = MINECRAFT_RESOURCES.to_string() + path.as_str();

                Artifact {
                    id: Some(key.clone()),
                    path: Some(path),
                    sha1: hash.to_string(),
                    size,
                    url,
                }
            })
            .collect::<Vec<_>>();


        Ok(artifacts)
    }

    async fn check_auth(&self, auth: &Account) -> Result<(), ManagerError>{
        let start = std::time::SystemTime::now();
        let now = start.duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        if now >= auth.auth_exp_time {
            auth_route::auther::refresh_oauth(&auth.auth).await?;
        }

        if now >= auth.mc_exp_time {
            auth_route::auther::auth_minecraft_token(&auth.auth).await?;
        }
        
        Ok(())
    }

    pub async fn load_version(&self, game: Game, minecraft_account: Account) -> Result<(), ManagerError> {
        let version = game.version;
        // Each game would contain a list of mods that need to be loaded across with resources.

        info!("Starting game {} ({}) with account {} and version {}", &game.name, &game.id, &minecraft_account.profile.name, version.to_string());
        let version_path = self.versions.join(&version.to_string());

        if !version_path.exists() {
            fs::create_dir(&version_path)?;
        }

        info!("Auth check");
        self.check_auth(&minecraft_account).await?;

        info!("Loading version {}", version.to_string());
        let manifest = self.get_manifest(&version_path, &version).await?;

        let asset_path = self.common.join("assets");
        let libraries_path = self.common.join("libs");
        let natives_temp_path = version_path.join("natives");

        // Version construct will include everything related to the version
        let construct = VersionConstruct {
            id: version,
            at: version_path,
            manifest,
            asset_path,
            libraries_path,
            natives_temp_path,
        };

        info!("Checking version...");
        self.check_version(&construct).await?;

        let instance_path = self.instances.join(&game.id);

        if !instance_path.exists() {
            fs::create_dir(&instance_path)?;
        }

        info!("Loading arguments...");
        let args = self.build_arguments(
            construct,
            minecraft_account,
            game,
            &instance_path,
        );

        info!("Initializing Minecraft process...");
        let handle = tokio::spawn(async move {
            info!("Running Minecraft");
            let (mut rx, _) = Command::new("java")
                .args(args)
                .current_dir(instance_path)
                .spawn()
                .expect("Failed to start java");

            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stderr(err) => error!("{}", err),
                    CommandEvent::Stdout(out) => info!("{}", out),
                    CommandEvent::Error(err) => error!("{}", err),
                    CommandEvent::Terminated(end) => info!("Terminated with code {:?}", end.code),
                    event => println!("Unknown event: {:?}", event),
                };
            }
        }.instrument(tracing::info_span!("MINECRAFT")));


        let _ = tokio::join!(handle);

        Ok(())
    }
}