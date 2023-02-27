use reqwest::Request;
use serde::de::DeserializeOwned;
use serde_json::json;

use super::{
    errors::AuthError,
    tokens::{MinecraftProfile, MinecraftToken, OAuthToken, XBLToken, XSTSToken},
    utils,
};

use futures::FutureExt;

const BASE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const CLIENT_ID: &str = "091170c6-c12e-4075-b7d0-05c916708c31";

const OAUTH_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBL_TOKEN_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_TOKEN_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_TOKEN_URL: &str =
    "https://api.minecraftservices.com/authentication/login_with_xbox";
const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

const SCOPE: &str = "XboxLive.signin offline_access";

/// Generates a URL to authenticate with Microsoft and returns the URL and the state.
///
/// # Example
///
/// ```
/// let (uri, state) = create_authentication_url("http://localhost:3000");
/// ```
pub fn create_authentication_url(redirect_uri: &str) -> (String, String) {
    let state = utils::next_state();
    let params = [
        ("response_type", "code"),
        ("client_id", CLIENT_ID),
        ("redirect_uri", redirect_uri),
        ("scope", SCOPE),
        ("response_mode", "query"),
        ("prompt", "select_account"),
        ("state", &state),
    ];

    let query = params
        .iter()
        .enumerate()
        .fold(String::new(), |mut acc, (i, (key, value))| {
            if i == 0 {
                acc.push('?');
            } else {
                acc.push('&');
            }

            acc.push_str(key);
            acc.push('=');
            acc.push_str(value);

            acc
        });

    (format!("{}{}", BASE_URL, query), state)
}

async fn extract_response<T, O>(
    client: reqwest::Client,
    request: Request,
    err_map: O,
) -> Result<T, AuthError>
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
        let result = response
            .json::<T>()
            .await
            .map_err(|err| err_map(err.to_string()))?;

        Ok(result)
    } else {
        let err = response
            .text()
            .await
            .map_err(|err| err_map(err.to_string()))?;
        let status_code = status.as_str();
        Err(err_map(format!(
            "Invalid response {}: {}",
            status_code, err
        )))
    }
}

async fn process_code(redirect_uri: &str, code: &str) -> Result<OAuthToken, AuthError> {
    let client = reqwest::Client::new();
    let request = client
        .post(OAUTH_TOKEN_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("scope", SCOPE),
            ("grant_type", "authorization_code"),
        ])
        .build()
        .expect("Failed to build request");

    extract_response(client, request, |err| AuthError::OAuthError(err)).await
}

async fn process_oauth_token(token: &OAuthToken) -> Result<XBLToken, AuthError> {
    let client = reqwest::Client::new();

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

async fn process_xbl_token(token: XBLToken) -> Result<XSTSToken, AuthError> {
    let token = token.token;

    let client = reqwest::Client::new();
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

async fn process_xsts_token(token: XSTSToken) -> Result<MinecraftToken, AuthError> {
    let client = reqwest::Client::new();
    if let Some((uhs, token)) = token.extract_tokens() {
        let token = format!("XBL3.0 x={};{}", uhs, token);
        let request = client
            .post(MINECRAFT_TOKEN_URL)
            .json(&json!({ "identityToken": token }))
            .build()
            .expect("Failed to build minecraft token request");

        extract_response(client, request, |err| AuthError::MinecraftTokenError(err)).await
    } else {
        Err(AuthError::MinecraftTokenError(
            "Failed to extract tokens from xsts token".to_string(),
        ))
    }
}

async fn process_minecraft_token(token: &MinecraftToken) -> Result<MinecraftProfile, AuthError> {
    let client = reqwest::Client::new();
    let request = client
        .get(MINECRAFT_PROFILE_URL)
        .bearer_auth(&token.access_token)
        .build()
        .expect("Failed to build minecraft profile request");

    extract_response(client, request, |_| AuthError::MissingMinecraftProfile).await
}

pub struct AuthParameters {
    pub oauth_token: OAuthToken,
    pub minecraft_token: MinecraftToken,
    pub minecraft_profile: MinecraftProfile,
}

pub async fn process_authentication_code(
    redirect_uri: &str,
    code: &str,
) -> Result<AuthParameters, AuthError> {
    let oauth_token = process_code(redirect_uri, code).await?;
    let minecraft_token = process_oauth_token(&oauth_token)
        .then(|res| async move {
            match res.map(|token| process_xbl_token(token)) {
                Ok(a) => a.await,
                Err(err) => Err(err),
            }
        })
        .then(|res| async move {
            match res.map(|token| process_xsts_token(token)) {
                Ok(a) => a.await,
                Err(err) => Err(err),
            }
        })
        .await?;

    let minecraft_profile = process_minecraft_token(&minecraft_token).await?;

    Ok(AuthParameters {
        oauth_token,
        minecraft_token,
        minecraft_profile,
    })
}
