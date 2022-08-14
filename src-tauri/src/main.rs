#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

mod auth_route;

use tauri::http::ResponseBuilder;
use auth_route::client;
use crate::auth_route::code_extractor::CodeExtractor;

#[tauri::command]
async fn auth_client(handle: tauri::AppHandle, window: tauri::Window) {
    client::start(handle, window).await;
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![auth_client])
        .register_uri_scheme_protocol("cognatize", |handle, req| {
            CodeExtractor::config(&handle, req.uri().to_string());

            Ok(ResponseBuilder::new()
                .status(200)
                .body("You can close this page".as_bytes().to_vec())
                .unwrap()
            )
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
