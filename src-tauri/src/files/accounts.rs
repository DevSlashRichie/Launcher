
use serde::{Serialize, Deserialize};
use crate::auth_route::tokens::{MinecraftProfile, MinecraftToken, OAuthToken};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AccountStorage {
    pub accounts: Vec<Account>,
    pub elected_account: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Account {
    pub profile: MinecraftProfile,
    pub mc: MinecraftToken,
    pub auth: OAuthToken,
}