#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

mod auth_route;
mod files;
mod asset_manager;

use tauri::http::ResponseBuilder;
use tauri::Manager;
use auth_route::auther;
use crate::asset_manager::version::VersionId;
use crate::auth_route::code_extractor::CodeExtractor;
use crate::files::accounts::{Account, AccountStorage};
use crate::files::settings::Settings;
use crate::files::storage::Storage;

const STORAGE_FOLDER: &str = ".cognatize";

#[tauri::command]
async fn add_account(handle: tauri::AppHandle, window: tauri::Window) {
    auther::authenticate(handle, window).await;
}

#[tauri::command]
fn remove_account(handle: tauri::AppHandle, account: u32) {
    let storage = handle.state::<Storage>().inner().extract();
    let mut storage = storage.write().unwrap();

    let store = &mut storage.settings.accounts.contents;

    let remove_elected = if let Some(account) = store.accounts.get(account as usize) {
        if let Some(elected) = &store.elected_account {
            elected == &account.uuid
        } else {
            false
        }
    } else {
        false
    };

    store.accounts.remove(account as usize);

    if remove_elected {
        store.elected_account = None;
    }

}

#[tauri::command]
fn get_accounts(handle: tauri::AppHandle) -> AccountStorage {
    let storage = handle.state::<Storage>().inner().extract();
    let storage = storage.read().unwrap();

    storage.settings.accounts.contents.clone()
}

#[tauri::command]
fn elect_account(handle: tauri::AppHandle, account: u32) {
    let storage = handle.state::<Storage>().inner().extract();
    let mut storage = storage.write().unwrap();
    let accounts = &mut storage.settings.accounts.contents;
    let account = accounts.accounts.get(account as usize);

    if let Some(account) = account {
        accounts.elected_account = Some(account.uuid.clone());
        storage.settings.accounts.save();
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![add_account, get_accounts, remove_account, elect_account])
        .register_uri_scheme_protocol("cognatize", |handle, req| {
            CodeExtractor::config(handle, req.uri().to_string());

            Ok(ResponseBuilder::new()
                .status(200)
                .body("You can close this page".as_bytes().to_vec())
                .unwrap()
            )
        })
        .setup(|app| {
            let storage = Storage::create(STORAGE_FOLDER)?;

            let st = storage.extract();
            tauri::async_runtime::block_on(async move {
                let res = st.read().unwrap().assets.check_version(VersionId::V1_19_2).await;

                println!("{:?}", res);
            });

            app.manage(storage);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
