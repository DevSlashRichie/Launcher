use crate::auth_route::code_processor;
use crate::auth_route::code_processor::CodeProcessor;
use crate::auth_route::errors::AuthError;
use crate::auth_route::tokens::{MinecraftProfile, MinecraftToken, OAuthToken};
use crate::{Account, Storage};
use serde::Serialize;
use tauri::{AppHandle, Manager, Window};

use super::auth_process::AuthParameters;
use super::{auth_process, code_listener};

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

async fn start_process(handle: &AppHandle, window: &Window) -> Result<(), AuthError> {
    window
        .emit(
            "auth:state",
            AuthStateEvent {
                message: "Please login to your account".to_string(),
                state: EventState::INFO,
            },
        )
        .ok();

    let (code, redirect_uri) = code_listener::create(&handle).await?;

    window
        .emit(
            "auth:state",
            AuthStateEvent {
                message: "Fetching your details".to_string(),
                state: EventState::INFO,
            },
        )
        .ok();

    let AuthParameters {
        minecraft_profile,
        minecraft_token,
        oauth_token,
    } = auth_process::process_authentication_code(&redirect_uri, &code).await?;

    window
        .emit(
            "auth:state",
            AuthStateEvent {
                message: minecraft_profile.name.clone(),
                state: EventState::DONE,
            },
        )
        .ok();

    let storage = handle.state::<Storage>().inner().extract();
    let mut storage = storage.write().unwrap();

    let store = &mut storage.settings.accounts.contents;

    // If an account with the same id already exists we remove it.
    //   In the big screen we are replacing it for a more updated token.
    if let Some(account_position) = store
        .accounts
        .iter()
        .position(|it| it.profile.id == minecraft_profile.id)
    {
        store.accounts.remove(account_position);
    }

    let id = minecraft_profile.id.clone();

    let start = std::time::SystemTime::now();
    let now = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let mc_exp_time = now + minecraft_token.expires_in;
    let auth_exp_time = now + oauth_token.expires_in;

    store.accounts.push(Account {
        auth: oauth_token,
        profile: minecraft_profile,
        mc: minecraft_token,
        mc_exp_time,
        auth_exp_time,
    });

    store.elected_account = Some(id);

    storage.settings.accounts.save().ok();

    Ok(())
}

pub async fn authenticate(handle: AppHandle, window: Window) {
    if let Err(err) = start_process(&handle, &window).await {
        println!("Error: {}", err);
        window
            .emit(
                "auth:state",
                AuthStateEvent {
                    message: err.to_string(),
                    state: EventState::ERROR,
                },
            )
            .ok();
    }
}

pub async fn refresh_oauth(auth_token: &OAuthToken) -> Result<OAuthToken, AuthError> {
    CodeProcessor::new().refresh_oauth(auth_token).await
}

pub async fn auth_minecraft_token(
    auth_token: &OAuthToken,
) -> Result<(MinecraftToken, MinecraftProfile), AuthError> {
    CodeProcessor::new().auth_minecraft_token(auth_token).await
}

pub async fn validate_token(auth: &mut Account) -> Result<bool, AuthError> {
    let start = std::time::SystemTime::now();
    let now = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let mut update = false;

    if now >= auth.auth_exp_time {
        let new_token = refresh_oauth(&auth.auth).await?;
        auth.auth = new_token;
        update = true;
    }

    if now >= auth.mc_exp_time {
        let (minecraft_token, minecraft_profile) = auth_minecraft_token(&auth.auth).await?;
        auth.mc = minecraft_token;
        auth.profile = minecraft_profile;
        update = true;
    }

    Ok(update)
}
