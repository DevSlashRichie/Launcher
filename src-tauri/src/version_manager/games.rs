use serde::{Deserialize, Serialize};
use crate::VersionId;

pub const AVAILABLE_GAMES: &[(&str, &str, VersionId); 1] = &[
    ("thebox_1.0", "The Box", VersionId::V1_19_2)
];


#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GameStorage {
    pub elected_game: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub version: VersionId,
}

impl Game {

    pub fn from_static((id, name, version): &(&str, &str, VersionId)) -> Game {
        Game {
            id: id.to_string(),
            name: name.to_string(),
            version: *version,
        }
    }

}