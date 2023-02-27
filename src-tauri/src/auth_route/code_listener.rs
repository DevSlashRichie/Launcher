use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use regex::Regex;
use tauri::{AppHandle, WindowBuilder};

use crate::{auth_route::auth_process, oauth_plugin::start};

use super::errors::AuthError;

const CODE_PARAMS_REGEX: &str = r"[&?]((\w+)=([\w\d\.-]+))";

/// Starts the code listener and returns the code and the state
/// This will block the current thread until the code is received after the user
/// has logged in inside the window which is opened by this function
///
/// The third paramter returned is the redirect uri which is used to listen for the code
/// You might want to have this parameter to verify the integrity of the code
///
///
/// # Example
/// ```
/// #[tauri::command]
/// async fn login(app_handle: AppHandle)  {
///     let (code, redirect_uri) = code_listener::create(&app_handle).await;
/// }
/// ```
pub async fn create(app_handle: &AppHandle) -> Result<(String, String), AuthError> {
    let code_state = Arc::new(RwLock::new(None));

    // should start be async?
    let copy = code_state.clone();
    let port = start(move |url| {
        copy.write().unwrap().replace(url);
    })
    .unwrap();

    let redirect_uri = format!("http://localhost:{}", port);
    let (url, state) = auth_process::create_authentication_url(&redirect_uri);

    let window = WindowBuilder::new(
        app_handle,
        "abcex",
        tauri::WindowUrl::External(url.parse().unwrap()),
    )
    .title("Iniciar sesión")
    .build()
    .unwrap();

    loop {
        let state = code_state.read().unwrap();
        if state.is_some() {
            break;
        }
    }

    window.close().unwrap();

    let complete_uri = code_state.read().unwrap().clone().unwrap();

    let (code, incoming_state) = extract_code(&complete_uri)?;

    if incoming_state != state {
        Err(AuthError::OAuthError("State is malformed.".into()))
    } else {
        Ok((code, redirect_uri))
    }
}

fn extract_code(uri: &str) -> Result<(String, String), AuthError> {
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

            Ok((code, state))
        }
    } else {
        Err(AuthError::InvalidRedirectUri)
    }
}
