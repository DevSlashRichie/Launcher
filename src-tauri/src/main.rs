#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

mod auth_route;

use tauri::http::ResponseBuilder;
use tauri::Manager;
use auth_route::client;

#[tauri::command]
async fn auth_client(handle: tauri::AppHandle) {
    client::start(handle).await;
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![auth_client])
        .register_uri_scheme_protocol("cognatize", |handle, req| {
            if let Some(window) = handle.get_window("auth") {
                window.trigger("code", Some(req.uri().to_string()));
            }

            Ok(ResponseBuilder::new()
                .status(200)
                .body("You can close this page".as_bytes().to_vec())
                .unwrap()
            )
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
