
use serde::{Serialize, Deserialize};
use crate::auth_route::tokens::{MinecraftProfile, MinecraftToken, OAuthToken};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct AccountStorage {
    pub accounts: Vec<Account>,
    pub elected_account: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    pub profile: MinecraftProfile,
    pub mc: MinecraftToken,
    pub auth: OAuthToken,
    pub mc_exp_time: u64,
    pub auth_exp_time: u64,
}

impl Account {
    
    pub fn is_mc_expired(&self) -> bool {
        self.mc_exp_time <= std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
    }
    
    pub fn is_auth_expired(&self) -> bool {
        self.auth_exp_time <= std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
    }
    
}