#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod auth_route;
mod files;
mod oauth_plugin;
mod version_manager;

use crate::auth_route::accounts::{Account, AccountStorage};
use crate::files::settings::Settings;
use crate::files::storage::Storage;
use crate::version_manager::games::{Game, AVAILABLE_GAMES};
use crate::version_manager::version::VersionId;
use auth_route::auther;
use tauri::Manager;
use tracing::{error, info};

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
            elected == &account.profile.id
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
async fn elect_account(handle: tauri::AppHandle, account: u32) {
    let storage = handle.state::<Storage>().inner().extract();
    let mut storage = storage.write().unwrap();
    let accounts = &mut storage.settings.accounts.contents;
    let account = accounts.accounts.get(account as usize);

    if let Some(account) = account {
        accounts.elected_account = Some(account.profile.id.clone());
        storage.settings.accounts.save();
    }
}

#[tauri::command]
fn get_games(handle: tauri::AppHandle) -> (Vec<Game>, Option<String>) {
    let storage = handle.state::<Storage>().inner().extract();
    let storage = storage.read().unwrap();

    let games = AVAILABLE_GAMES
        .iter()
        .map(|entry| Game::from_static(entry))
        .collect();
    (games, storage.settings.games.contents.elected_game.clone())
}

#[tauri::command]
fn pick_game(handle: tauri::AppHandle, id: String) {
    let storage = handle.state::<Storage>().inner().extract();
    let mut storage = storage.write().unwrap();
    let game = AVAILABLE_GAMES.iter().find(|game| game.0 == id);

    if let Some(game) = game {
        storage.settings.games.contents.elected_game = Some(game.0.to_string());
        storage.settings.games.save();
    }
}

#[tauri::command]
async fn start_game(handle: tauri::AppHandle) {
    let source = handle.state::<Storage>().inner();

    let data = {
        let storage = source.extract();
        let storage = storage.read().unwrap();

        let assets = storage.assets.clone();

        let accounts = &storage.settings.accounts.contents;
        let games = &storage.settings.games.contents;

        // Awful "hack" to get the elected account
        if let Some(account) = &accounts.elected_account {
            if let Some(account) = accounts.accounts.iter().find(|x| &x.profile.id == account) {
                if let Some(game) = &games.elected_game {
                    if let Some(game) = AVAILABLE_GAMES.iter().find(|it| it.0 == game.as_str()) {
                        let game = Game::from_static(game);
                        Some((account.clone(), assets, game))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some((mut account, assets, game)) = data {
        let should_save = auther::validate_token(&mut account).await;

        {
            match should_save {
                Err(err) => error!("Error while validating token: {}", err),
                Ok(should_save) => {
                    if should_save {
                        info!("Token has been updated");
                        let storage = source.extract();
                        let mut storage = storage.write().unwrap();
                        let pos = storage
                            .settings
                            .accounts
                            .contents
                            .accounts
                            .iter_mut()
                            .position(|x| x.profile.id == account.profile.id)
                            .unwrap();
                        storage.settings.accounts.contents.accounts[pos] = account.clone();
                        storage.settings.accounts.save();
                    }
                }
            }
        }

        let res = assets.load_version(game, account).await;

        if let Err(err) = res {
            error!("{:?}", err);
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(oauth_plugin::init())
        .invoke_handler(tauri::generate_handler![
            add_account,
            get_accounts,
            remove_account,
            elect_account,
            start_game,
            get_games,
            pick_game
        ])
        .setup(|app| {
            info!("Initializing Launcher");
            let storage = Storage::create(STORAGE_FOLDER)?;

            info!("Storage initialized");
            app.manage(storage);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
