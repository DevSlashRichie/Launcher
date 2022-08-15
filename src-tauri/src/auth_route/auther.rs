use std::borrow::BorrowMut;
use std::ops::DerefMut;
use tauri::{AppHandle, Manager, Window};
use crate::auth_route::{code_processor};
use serde::Serialize;
use crate::auth_route::code_processor::CodeProcessor;
use crate::auth_route::errors::AuthError;
use crate::{Account, CodeExtractor, Storage};

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

pub async fn authenticate(handle: AppHandle, window: Window) {
    let (auth_url, state) = code_processor::authenticate_url();

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
            CodeProcessor::with(code).await
        } else {
            Err(AuthError::InvalidState)
        }
    } else {
        Err(AuthError::MissingMinecraftProfile)
    };

    match minecraft_profile {
        Ok((profile, token)) => {
            window.emit("auth:state", AuthStateEvent {
                message: profile.name.clone(),
                state: EventState::DONE,
            }).ok();

            let storage = handle.state::<Storage>().inner().extract();
            let mut storage = storage.write().unwrap();

            storage.settings.accounts.contents.push(Account {
                username: profile.name,
                uuid: profile.id,
                access_token: token.access_token
            });

            storage.settings.accounts.save();
        }

        Err(err) => {
            window.emit("auth:state", AuthStateEvent {
                message: err.to_string(),
                state: EventState::ERROR,
            }).ok();
        }
    };

}