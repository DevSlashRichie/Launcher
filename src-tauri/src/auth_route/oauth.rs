use reqwest::{Request, Response};
use serde::de::DeserializeOwned;
use serde_json::json;
use crate::auth_route::errors::AuthError;
use crate::auth_route::tokens::{CodeToken, MinecraftProfile, MinecraftToken, OAuthToken, XBLToken, XSTSToken};
use crate::client::{SCOPE, CLIENT_ID, REDIRECT_URI};

const OAUTH_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBL_TOKEN_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_TOKEN_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_TOKEN_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

async fn extract_response<T, O>(client: &reqwest::Client, request: Request, err_map: O) -> Result<T, AuthError>
    where
        T: DeserializeOwned,
        O: Fn(String) -> AuthError,
{
    let response = client
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

pub async fn get_auth_token(client: &reqwest::Client, code: CodeToken) -> Result<OAuthToken, AuthError> {
    let request = client
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

    extract_response(client, request, |err| AuthError::XBLError(err)).await
}

pub async fn get_xbl_token(client: &reqwest::Client, token: OAuthToken) -> Result<XBLToken, AuthError> {
    let token = format!("d={}", token.access_token);
    let request = client
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


    extract_response(client, request, |err| AuthError::XBLError(err)).await
}

pub async fn get_xsts_token(client: &reqwest::Client, token: XBLToken) -> Result<XSTSToken, AuthError> {
    let token = token.extract_token();

    let request = client
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

    extract_response(client, request, |err| AuthError::XSTSError(err)).await
}

pub async fn get_minecraft_token(client: &reqwest::Client, token: XSTSToken) -> Result<MinecraftToken, AuthError> {
    if let Some((uhs, token)) = token.extract_tokens() {
        let token = format!("XBL3.0 x={};{}", uhs, token);
        let request = client
            .post(MINECRAFT_TOKEN_URL)
            .json(&json!({
                "identityToken": token
            }))
            .build()
            .expect("Failed to build minecraft token request");

        extract_response(client, request, |err| AuthError::MinecraftTokenError(err)).await
    } else {
        Err(AuthError::MinecraftTokenError("Failed to extract tokens from xsts token".to_string()))
    }
}

pub async fn get_minecraft_profile(client: &reqwest::Client, token: MinecraftToken) -> Result<MinecraftProfile, AuthError> {
let request = client
        .get(MINECRAFT_PROFILE_URL)
        .bearer_auth(&token.access_token)
        .build()
        .expect("Failed to build minecraft profile request");

    extract_response(client, request, |err| AuthError::MinecraftTokenError(err)).await
}