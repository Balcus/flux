use crate::{AppState, models::{BranchInfo, RepositoryInfo}};
use flux_core::{error::{ConfigError, RefsError}, internals::repository::Repository};
use tauri::State;

#[tauri::command]
pub fn open_repository(path: String, state: State<AppState>) -> Result<RepositoryInfo, String> {
    let repo = Repository::open(Some(path)).map_err(|err| err.to_string())?;
    let info = RepositoryInfo::from_repo(&repo)?;
    let mut repo_lock = state.repository.lock().unwrap();
    *repo_lock = Some(repo);
    Ok(info)
}

#[tauri::command]
pub fn close_repository(state: State<AppState>) -> Result<(), String> {
    let mut repo_lock = state.repository.lock().unwrap();
    *repo_lock = None;
    Ok(())
}

#[tauri::command]
pub fn get_repository_info(state: State<AppState>) -> Result<RepositoryInfo, String> {
    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_ref()
        .ok_or_else(|| "No repository open".to_string())?;
    RepositoryInfo::from_repo(repo)
}

#[tauri::command]
pub fn update_user_config(
    user_name: String,
    user_email: String,
    state: State<AppState>,
) -> Result<(), String> {
    let mut repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_mut()
        .ok_or_else(|| "No repository open".to_string())?;
    
    repo.config.set("user_name".to_string(), user_name).map_err(|e: ConfigError| e.to_string())?;
    repo.config.set("user_email".to_string(), user_email).map_err(|e: ConfigError| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn update_origin(origin: String, state: State<AppState>) -> Result<(), String> {
    let mut repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_mut()
        .ok_or_else(|| "No repository open".to_string())?;
    
    repo.config.set("origin".to_string(), origin).map_err(|e: ConfigError| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn get_branches(state: State<AppState>) -> Result<Vec<BranchInfo>, String> {
    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_ref()
        .ok_or_else(|| "No repository open".to_string())?;
    
    let current = repo.refs.current_branch().map_err(|e: RefsError| e.to_string())?;
    let mut branches: Vec<BranchInfo> = repo
        .refs
        .branch_names()
        .into_iter()
        .map(|name| BranchInfo {
            is_current: name == current,
            name,
        })
        .collect();
    
    branches.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(branches)
}