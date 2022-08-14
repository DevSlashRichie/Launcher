use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::{Duration};
use tauri::{AppHandle, Window, WindowBuilder, WindowUrl};
use crate::auth_route::{oauth, utils};
use std::sync::atomic::Ordering;
use serde::Serialize;
use crate::auth_route::errors::AuthError;
use crate::auth_route::tokens::{CodeToken, MinecraftProfile};
use crate::CodeExtractor;

pub const CLIENT_ID: (&str, &str) = ("client_id", "091170c6-c12e-4075-b7d0-05c916708c31");
pub const REDIRECT_URI: (&str, &str) = ("redirect_uri", "https://login.microsoftonline.com/common/oauth2/nativeclient");
pub const SCOPE: (&str, &str) = ("scope", "XboxLive.signin");

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

#[derive(Serialize, Clone)]
enum EventState {
    INFO,
    DONE,
    ERROR,
}

#[derive(Serialize, Clone)]
struct AuthStateEvent {
    state: EventState,
    message: String,
}

/// Returns (uri, state)
fn build_url() -> (String, String) {
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

async fn process_code(code: CodeToken) -> Result<MinecraftProfile, AuthError> {
    let client = reqwest::Client::new();

    let auth_token = oauth::get_auth_token(&client, code).await?;
    let xbl_token = oauth::get_xbl_token(&client, auth_token).await?;
    let xsts_token = oauth::get_xsts_token(&client, xbl_token).await?;
    let minecraft_token = oauth::get_minecraft_token(&client, xsts_token).await?;
    let minecraft_profile = oauth::get_minecraft_profile(&client, minecraft_token).await?;

    Ok(minecraft_profile)
}

pub async fn start(handle: AppHandle, window: Window) {
    let (auth_url, state) = build_url();

    let code_extractor = CodeExtractor::open(&handle, "Iniciar sesión", &auth_url);

    window.emit("auth:state", AuthStateEvent {
        message: "Please login to your account".to_string(),
        state: EventState::INFO,
    }).ok();

    let code = code_extractor.fetch().await;

    window.emit("auth:state", AuthStateEvent {
        message: "Fetching your details".to_string(),
        state: EventState::INFO,
    }).ok();

    let minecraft_profile = if let Some(code) = code {
        if state == code.state {
            process_code(code).await
        } else {
            Err(AuthError::InvalidState)
        }
    } else {
        Err(AuthError::MissingMinecraftProfile)
    };

    match minecraft_profile {
        Ok(profile) => {
            window.emit("auth:state", AuthStateEvent {
                message: profile.name,
                state: EventState::DONE,
            }).ok();
        }

        Err(err) => {
            window.emit("auth:state", AuthStateEvent {
                message: err.to_string(),
                state: EventState::ERROR,
            }).ok();
        }
    };

}