#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

mod auth_route;
mod files;

use tauri::http::ResponseBuilder;
use tauri::Manager;
use auth_route::auther;
use crate::auth_route::code_extractor::CodeExtractor;
use crate::files::accounts::Account;
use crate::files::settings::Settings;
use crate::files::storage::Storage;

const STORAGE_FOLDER: &str = ".cognatize";

#[tauri::command]
async fn auth_client(handle: tauri::AppHandle, window: tauri::Window) {
    auther::authenticate(handle, window).await;
}

#[tauri::command]
fn get_accounts(handle: tauri::AppHandle) -> Vec<Account> {
    let storage = handle.state::<Storage>().inner().extract();
    let storage = storage.read().unwrap();

    storage.settings.accounts.contents.clone()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![auth_client, get_accounts])
        .register_uri_scheme_protocol("cognatize", |handle, req| {
            CodeExtractor::config(&handle, req.uri().to_string());

            Ok(ResponseBuilder::new()
                .status(200)
                .body("You can close this page".as_bytes().to_vec())
                .unwrap()
            )
        })
        .setup(|app| {
            let mut storage = Storage::create(STORAGE_FOLDER)?;

            app.manage(storage);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
