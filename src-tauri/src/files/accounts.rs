
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AccountStorage {
    pub accounts: Vec<Account>,
    pub elected_account: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Account {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
}