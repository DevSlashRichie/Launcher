use reqwest::{Request};
use serde::de::DeserializeOwned;
use serde_json::json;
use crate::auth_route::errors::AuthError;
use crate::auth_route::tokens::{CodeToken, MinecraftProfile, MinecraftToken, OAuthToken, XBLToken, XSTSToken};
use crate::auth_route::utils;

const CLIENT_ID: (&str, &str) = ("client_id", "091170c6-c12e-4075-b7d0-05c916708c31");
const REDIRECT_URI: (&str, &str) = ("redirect_uri", "https://login.microsoftonline.com/common/oauth2/nativeclient");
const SCOPE: (&str, &str) = ("scope", "XboxLive.signin offline_access");

const BASE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const PARAMS: &[(&str, &str)] = &[
    ("response_type", "code"),
    CLIENT_ID,
    REDIRECT_URI,
    SCOPE,
    ("response_mode", "query"),
    ("prompt", "login"),
    ("state", ""),
];

const OAUTH_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBL_TOKEN_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_TOKEN_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_TOKEN_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

/// Returns (uri, state)
pub fn authenticate_url() -> (String, String) {
    let mut params = String::new();
    let state = utils::next_state();
    for (i, (key, value)) in PARAMS.iter().enumerate() {
        if i == 0 {
            params.push('?');
        } else {
            params.push('&');
        }

        params.push_str(key);
        params.push('=');

        if value.len() == 0 {
            match *key {
                "state" => params.push_str(&state),
                _ => (),
            }
        } else {
            params.push_str(value);
        }
    }

    (format!("{}{}", BASE_URL, params), state)
}

pub struct CodeProcessor {
    client: reqwest::Client,
}

impl CodeProcessor {
    pub fn new() -> Self {
        Self { client: reqwest::Client::new() }
    }

    pub async fn with(code: CodeToken) -> Result<(MinecraftProfile, MinecraftToken, OAuthToken), AuthError> {
        Self::new().process(code).await
    }

    pub async fn process(&self, code: CodeToken) -> Result<(MinecraftProfile, MinecraftToken, OAuthToken), AuthError> {

        let auth_token = self.get_auth_token(code).await?;
        let xbl_token = self.get_xbl_token(auth_token.clone()).await?;
        let xsts_token = self.get_xsts_token(xbl_token).await?;
        let minecraft_token = self.get_minecraft_token(xsts_token).await?;
        let minecraft_profile = self.get_minecraft_profile(&minecraft_token).await?;

        Ok((minecraft_profile, minecraft_token, auth_token))
    }

    pub async fn refresh_oauth(&self, auth_token: OAuthToken) -> Result<OAuthToken, AuthError> {
        let request = self.client
            .post(OAUTH_TOKEN_URL)
            .form(&[
                CLIENT_ID,
                SCOPE,
                ("refresh_token", &auth_token.refresh_token),
                ("grant_type", "refresh_token"),
            ])
            .build()
            .expect("Failed to build code exchange request");

        self.extract_response(request, |err| AuthError::XBLError(err)).await
    }

    pub async fn auth_minecraft_token(&self, auth_token: OAuthToken) -> Result<(MinecraftToken, MinecraftProfile), AuthError>{
        let xbl_token = self.get_xbl_token(auth_token.clone()).await?;
        let xsts_token = self.get_xsts_token(xbl_token).await?;
        let minecraft_token = self.get_minecraft_token(xsts_token).await?;
        let minecraft_profile = self.get_minecraft_profile(&minecraft_token).await?;

        Ok((minecraft_token, minecraft_profile))
    }

    async fn get_auth_token(&self, code: CodeToken) -> Result<OAuthToken, AuthError> {
        let request = self.client
            .post(OAUTH_TOKEN_URL)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", &code.code),
                REDIRECT_URI,
                CLIENT_ID,
                SCOPE,
            ])
            .build()
            .expect("Failed to build code exchange request");

        self.extract_response(request, |err| AuthError::XBLError(err)).await
    }

    async fn get_xbl_token(&self, token: OAuthToken) -> Result<XBLToken, AuthError> {
        let token = format!("d={}", token.access_token);
        let request = self.client
            .post(XBL_TOKEN_URL)
            .json(&json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": token
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        }))
            .build()
            .expect("Failed to build xbl token request");


        self.extract_response(request, |err| AuthError::XBLError(err)).await
    }

    async fn get_xsts_token(&self, token: XBLToken) -> Result<XSTSToken, AuthError> {
        let token = token.extract_token();

        let request = self.client
            .post(XSTS_TOKEN_URL)
            .json(&json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
            .build()
            .expect("Failed to build xsts token request");

        self.extract_response(request, |err| AuthError::XSTSError(err)).await
    }

    async fn get_minecraft_token(&self, token: XSTSToken) -> Result<MinecraftToken, AuthError> {
        if let Some((uhs, token)) = token.extract_tokens() {
            let token = format!("XBL3.0 x={};{}", uhs, token);
            let request = self.client
                .post(MINECRAFT_TOKEN_URL)
                .json(&json!({
                "identityToken": token
            }))
                .build()
                .expect("Failed to build minecraft token request");

            self.extract_response(request, |err| AuthError::MinecraftTokenError(err)).await
        } else {
            Err(AuthError::MinecraftTokenError("Failed to extract tokens from xsts token".to_string()))
        }
    }

    pub async fn get_minecraft_profile(&self, token: &MinecraftToken) -> Result<MinecraftProfile, AuthError> {
        let request = self.client
            .get(MINECRAFT_PROFILE_URL)
            .bearer_auth(&token.access_token)
            .build()
            .expect("Failed to build minecraft profile request");

        self.extract_response(request, |err| AuthError::MinecraftTokenError(err)).await
    }

    async fn extract_response<T, O>(&self, request: Request, err_map: O) -> Result<T, AuthError>
        where
            T: DeserializeOwned,
            O: Fn(String) -> AuthError,
    {
        let response = self.client
            .execute(request)
            .await
            .map_err(|err| err_map(err.to_string()))?;

        let status = response.status();
        if status.is_success() {
            let result = response.json::<T>()
                .await
                .map_err(|err| err_map(err.to_string()))?;

            Ok(result)
        } else {
            let err = response.text().await
                .map_err(|err| err_map(err.to_string()))?;
            let status_code = status.as_str();
            Err(err_map(format!("Invalid response {}: {}", status_code, err)))
        }
    }
}