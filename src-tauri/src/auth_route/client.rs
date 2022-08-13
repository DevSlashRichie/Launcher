use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::{Duration};
use tauri::{AppHandle, WindowBuilder, WindowUrl};
use crate::auth_route::{oauth, utils};

pub const CLIENT_ID: (&str, &str) = ("client_id", "091170c6-c12e-4075-b7d0-05c916708c31");
pub const REDIRECT_URI: (&str, &str) = ("redirect_uri", "https://login.microsoftonline.com/common/oauth2/nativeclient");
pub const SCOPE: (&str, &str) = ("scope", "User.Read");

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

pub async fn start(handle: AppHandle) {
    let (auth_url, state) = build_url();

    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    let window = WindowBuilder::new(&handle, "auth", WindowUrl::App("index.html".parse().unwrap()))
        .title("Auth")
        .build()
        .unwrap();

    // We need to redirect it because the only way we can use custom schema is by using an App URL
    window.eval(format!("location.replace('{}')", auth_url).as_str()).ok();

    let window_clone = window.clone();
    window.listen("code", move |ev| {
        if let Some(code) = ev.payload() {
            let code = oauth::CodeResponse::from_uri(code);
            println!("{:?}", code);

            stop_clone.store(true, std::sync::atomic::Ordering::Relaxed);
            window_clone.close().ok();
        }
    });

    loop {
        if stop.load(std::sync::atomic::Ordering::Relaxed) {
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
}