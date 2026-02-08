// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use flux_core::internals::repository::Repository;

mod commands;
mod models;

pub struct AppState {
    repository: Mutex<Option<Repository>>
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            repository: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::open_repository,
            commands::close_repository,
            commands::get_repository_info,
            commands::update_user_config,
            commands::update_origin,
            commands::get_branches,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
