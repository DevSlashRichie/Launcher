use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::{Duration};
use tauri::{AppHandle, WindowBuilder, WindowUrl};
use crate::auth_route::{oauth, utils};
use std::sync::atomic::Ordering;
use crate::auth_route::tokens::CodeToken;

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

async fn process_code(code: CodeToken) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    
    let auth_token = oauth::get_auth_token(&client, code).await?;
    let xbl_token = oauth::get_xbl_token(&client, auth_token).await?;
    let xsts_token = oauth::get_xsts_token(&client, xbl_token).await?;
    let minecraft_token = oauth::get_minecraft_token(&client, xsts_token).await?;
    let minecraft_profile = oauth::get_minecraft_profile(&client, minecraft_token).await?;
    println!("{:?}", minecraft_profile);

    Ok(())
}

pub async fn start(handle: AppHandle) {
    let (auth_url, state) = build_url();

    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();

    let tx = Arc::new(Mutex::new(None));
    let tx_clone = tx.clone();

    let window = WindowBuilder::new(&handle, "auth", WindowUrl::App("index.html".parse().unwrap()))
        .title("Auth")
        .build()
        .unwrap();

    // We need to redirect it because the only way we can use custom schema is by using an App URL
    window.eval(format!("location.replace('{}')", auth_url).as_str()).ok();

    let window_clone = window.clone();
    let event_handler = window.listen("code", move |ev| {
        if let Some(code) = ev.payload() {
            let code = code.to_string();

            if CodeToken::uri_includes_code(code.as_str()) {
                stop_clone.store(true, Ordering::Relaxed);
                window_clone.close().ok();

                if let Ok(mut tx) = tx_clone.try_lock() {
                    tx.replace(code);
                }
            }
        }
    });

    loop {
        if stop.load(Ordering::Relaxed) {
            window.unlisten(event_handler);
            break;
        }

        let code = "\
            if(location.href.includes('code=')) {
              location.replace('cognatize://' + location.search)
            }
        ";

        window.eval(code).ok();
        thread::sleep(Duration::from_millis(1000));
    }

    let code = if let Ok(code) = tx.lock() {
        if let Some(code) = code.as_ref() {
            Some(code.clone())
        } else {
            None
        }
    } else {
        None
    };

    if let Some(code) = code {
        if let Ok(code) = CodeToken::from_uri(code.as_str()) {
            
            if state != code.state {
                println!("State mismatch");
                return;
            }
            
            match process_code(code).await {
                Ok(_) => (),
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        }
    }
}