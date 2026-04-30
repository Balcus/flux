// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use desktop_app_lib::{commands, models::app_state::AppState};
use std::sync::Mutex;

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
            commands::get_graph,
            commands::add,
            commands::rm,
            commands::reset_soft,
            commands::reset_hard,
            commands::restore,
            commands::get_diff,
            commands::commit,
            commands::switch_branch,
            commands::delete_branch,
            commands::create_branch,
            commands::get_tree_changes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
