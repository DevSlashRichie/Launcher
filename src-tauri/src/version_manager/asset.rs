use std::fmt::Formatter;
use std::iter::Map;
use std::path::PathBuf;
use regex::Regex;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde::de::{Error, MapAccess};
use serde_json::{Value};

// ROOT
#[derive(Serialize, Deserialize, Debug)]
pub struct VersionManifest {
    pub id: String,
    pub libraries: Vec<Library>,
    pub downloads: Downloads,
    #[serde(rename = "assetIndex")]
    pub asset_index: Artifact,
    pub assets: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    pub arguments: Arguments,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Arguments {
    pub game: Vec<Argument>,
    pub jvm: Vec<Argument>,
}

#[derive(Serialize, Debug)]
pub enum Argument {
    Plain(String),
    WithRules {
        value: ArgumentValue,
        rules: Vec<Rule>,
    },
}

#[derive(Serialize, Debug)]
pub enum ArgumentValue {
    String(String),
    Array(Vec<String>),
}

impl<'de> de::Deserialize<'de> for Argument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct ArgumentVisitor;

        impl<'de> de::Visitor<'de> for ArgumentVisitor {
            type Value = Argument;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("Argument")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
                Ok(Argument::Plain(v.to_string()))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                if let Some(_) = map.next_key::<String>()? {
                    let rules: Vec<Rule> = map.next_value()?;

                    if let Some(_) = map.next_key::<String>()? {
                        let raw_value: Value = map.next_value()?;
                        let value = if raw_value.is_string() {
                            Ok(ArgumentValue::String(raw_value.as_str().unwrap().to_string()))
                        } else if raw_value.is_array() {
                            Ok(ArgumentValue::Array(raw_value.as_array().unwrap().iter()
                                .map(|x| x.as_str().unwrap().to_string())
                                .collect::<Vec<_>>()))
                        } else {
                            Err(Error::invalid_type(de::Unexpected::Other("not a string or array"), &"string or array"))?
                        }?;

                        Ok(Argument::WithRules {
                            value,
                            rules,
                        })
                    } else {
                        return Err(Error::missing_field("value"));
                    }
                } else {
                    Err(Error::missing_field("rules"))
                }
            }
        }

        deserializer.deserialize_any(ArgumentVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Downloads {
    pub client: Artifact,
    pub server: Artifact,
}

// ASSETS

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Artifact {
    pub id: Option<String>,
    pub path: Option<String>,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

impl Artifact {

    pub fn id(&self) -> String {
        if let Some(id) = self.id.as_ref() {
            id.clone()
        } else {
            self.file_name().to_string()
        }
    }

    pub fn file_name(&self) -> &str {
        self.url.split("/").last().unwrap()
    }

    pub fn derive_path(&self, at: &PathBuf) -> PathBuf {
        let file_name = self.file_name();

        if let Some(path) = &self.path {
            at.join(path)
        } else {
            at.join(file_name)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Os {
    pub name: Option<String>,
    pub version: Option<String>,
    pub arch: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    pub os: Option<Os>,
    pub action: String,
}

impl Rule {
    pub fn is_valid(&self) -> bool {
        if let Some(os) = &self.os {
            if let Some(name) = &os.name {
                let current_os = std::env::consts::OS;
                name == current_os && self.action == "allow"
            } else if let Some(arch) = &os.arch {
                let current_arch = std::env::consts::ARCH;
                arch == current_arch && self.action == "allow"
            } else if let Some(ver) = &os.version {
                let regex = Regex::new(ver)
                    .expect("Failed to compile version regex");
                let version = os_info::get().version().to_string();
                regex.is_match(&version) && self.action == "allow"
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    pub name: String,
    pub downloads: LibraryDownloads,
    pub rules: Option<Vec<Rule>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LibraryDownloads {
    pub artifact: Artifact,
}

// VERSIONS

#[derive(Serialize, Deserialize)]
pub struct Version {
    pub id: String,
    pub r#type: String,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct VersionLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Serialize, Deserialize)]
pub struct VersionsManifest {
    pub latest: VersionLatest,
    pub versions: Vec<Version>,
}
