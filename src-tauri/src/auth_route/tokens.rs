use crate::auth_route::errors::AuthError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const CODE_PARAMS_REGEX: &str = r"[&?]((\w+)=([\w\d\.-]+))";

pub enum Token {
    Code(CodeToken),
    Oauth(OAuthToken),
    XBL(XBLToken),
    XSTS,
}

#[derive(Debug, Clone)]
pub struct CodeToken {
    pub(crate) code: String,
    pub state: String,
}

impl CodeToken {
    pub fn new(code: String, state: String) -> Self {
        Self { code, state }
    }

    pub fn from_uri(uri: &str) -> Result<Self, AuthError> {
        let regex = Regex::new(CODE_PARAMS_REGEX).unwrap();
        if regex.is_match(uri) {
            // extract code and state
            let extracted_params = regex.captures_iter(uri);

            let mut params = extracted_params
                .map(|param| {
                    let key = param[2].to_string();
                    let value = param[3].to_string();
                    (key, value)
                })
                .collect::<HashMap<String, String>>();

            if !params.contains_key("code") || !params.contains_key("state") {
                Err(AuthError::OAuthError("Invalid Parameters".into()))
            } else {
                let code = params.remove_entry("code").unwrap().1;
                let state = params.remove_entry("state").unwrap().1;

                let entity = Self::new(code, state);
                Ok(entity)
            }
        } else {
            Err(AuthError::InvalidRedirectUri)
        }
    }

    pub fn uri_includes_code(uri: &str) -> bool {
        let regex = Regex::new(CODE_PARAMS_REGEX).unwrap();
        regex.is_match(uri)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    scope: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct XBLToken {
    #[serde(rename = "IssueInstant")]
    issue_instant: String,
    #[serde(rename = "NotAfter")]
    not_after: String,
    #[serde(rename = "Token")]
    pub token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: TokenDisplayClaims,
}

impl XBLToken {
    pub fn extract_token(self) -> String {
        self.token
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenDisplayClaims {
    xui: Vec<UserHash>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserHash {
    uhs: String,
}

impl Into<String> for UserHash {
    fn into(self) -> String {
        self.uhs
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct XSTSToken {
    #[serde(rename = "IssueInstant")]
    issue_instant: String,
    #[serde(rename = "NotAfter")]
    not_after: String,
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: TokenDisplayClaims,
}

impl XSTSToken {
    /// RETURNS (uhs, token)
    pub fn extract_tokens(self) -> Option<(String, String)> {
        if let Some(token) = self.display_claims.xui.first() {
            Some((token.uhs.clone(), self.token))
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftToken {
    username: String,
    pub access_token: String,
    token_type: String,
    pub expires_in: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
}
