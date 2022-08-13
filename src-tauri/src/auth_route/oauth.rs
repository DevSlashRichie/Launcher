use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::auth_route::errors::AuthError;
use crate::client::{SCOPE, CLIENT_ID, REDIRECT_URI};

const OAUTH_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const PARAMS_REGEX: &str = r"[&?]((\w+)=([\w\d\.-]+))";


#[derive(Serialize, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    expires_in: u64,
    scope: String,
    refresh_token: String,
    id_token: String,
}

#[derive(Debug)]
pub struct CodeResponse {
    code: String,
    state: String,
}

impl CodeResponse {
    pub fn new(code: String, state: String) -> Self {
        CodeResponse { code, state }
    }

    pub fn from_uri(uri: &str) -> Result<Self, AuthError> {

        let regex = Regex::new(PARAMS_REGEX).unwrap();
        if regex.is_match(uri) {
            // extract code and state
            let extracted_params = regex
                .captures_iter(uri);

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

                let entity = CodeResponse::new(code, state);
                Ok(entity)
            }
        } else {
            Err(AuthError::InvalidRedirectUri)
        }
        
    }

    async fn exchange_code(&self, client: reqwest::Client) -> anyhow::Result<OAuthTokenResponse> {
        let request = client
            .post(OAUTH_TOKEN_URL)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", self.code.as_str()),
                REDIRECT_URI,
                CLIENT_ID,
                SCOPE,
            ])
            .build()
            .expect("Failed to build code exchange request");

        let response = client
            .execute(request)
            .await?
            .json::<OAuthTokenResponse>()
            .await?;

        Ok(response)
    }

}
