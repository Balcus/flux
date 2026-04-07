use std::path::PathBuf;

use crate::models::{
    app_state::AppState, branch_info::BranchInfo, commit_info::CommitInfo,
    repository_info::RepositoryInfo, status_info::{StagedFile, StatusInfo},
};
use flux_core::{
    commands::add::AddCommand, database::{commit::Commit, database::Database}, error::{ConfigError, RefsError}, internals::repository::Repository, status::status_impl::{ChangeType, Status}
};
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

    repo.config
        .set("user_name".to_string(), user_name)
        .map_err(|e: ConfigError| e.to_string())?;
    repo.config
        .set("user_email".to_string(), user_email)
        .map_err(|e: ConfigError| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_origin(origin: String, state: State<AppState>) -> Result<(), String> {
    let mut repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_mut()
        .ok_or_else(|| "No repository open".to_string())?;

    repo.config
        .set("origin".to_string(), origin)
        .map_err(|e: ConfigError| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_branches(state: State<AppState>) -> Result<Vec<BranchInfo>, String> {
    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_ref()
        .ok_or_else(|| "No repository opened".to_string())?;

    let current = repo
        .refs
        .current_branch()
        .map_err(|e: RefsError| e.to_string())?;
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

#[tauri::command]
pub fn get_commits(state: State<AppState>) -> Result<Vec<CommitInfo>, String> {
    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_ref()
        .ok_or_else(|| "No repository open".to_string())?;
    let db = Database::open(repo.flux_dir.clone());

    let mut commits_map: std::collections::HashMap<String, CommitInfo> =
        std::collections::HashMap::new();
    let mut branch_entries: Vec<(&String, &String)> = repo.refs.branches.iter().collect();

    branch_entries.sort_by_key(|(name, _)| if *name == "main" { 0 } else { 1 });

    for (branch_name, tip_hash) in branch_entries {
        let mut current_hash = Some(tip_hash.clone());
        while let Some(hash) = current_hash {
            let obj = match db.read_object(&hash) {
                Ok(o) => o,
                Err(_) => break,
            };
            if let Some(commit) = obj.as_any().downcast_ref::<Commit>() {
                let parent = commit.parent_hash().map(String::from);
                commits_map
                    .entry(hash.clone())
                    .or_insert_with(|| CommitInfo {
                        id: hash.clone(),
                        message: commit.message.clone(),
                        author: commit.author.clone(),
                        parent: parent.clone(),
                        branch: branch_name.clone(),
                    });
                current_hash = parent;
            } else {
                break;
            }
        }
    }

    let mut sorted_list = Vec::new();
    let mut visited = std::collections::HashSet::new();

    fn visit(
        id: &str,
        map: &std::collections::HashMap<String, CommitInfo>,
        visited: &mut std::collections::HashSet<String>,
        list: &mut Vec<CommitInfo>,
    ) {
        if visited.contains(id) || !map.contains_key(id) {
            return;
        }
        visited.insert(id.to_string());
        let commit = &map[id];
        if let Some(ref parent_id) = commit.parent {
            visit(parent_id, map, visited, list);
        }
        list.push(commit.clone());
    }

    let mut keys: Vec<String> = commits_map.keys().cloned().collect();
    keys.sort();

    for id in keys {
        visit(&id, &commits_map, &mut visited, &mut sorted_list);
    }

    Ok(sorted_list)
}

#[tauri::command]
pub fn get_status(state: State<AppState>) -> Result<StatusInfo, String> {
    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_ref()
        .ok_or_else(|| "No repository open".to_string())?;

    let status = Status::new(repo).map_err(|e| e.to_string())?;

    let staged = status
        .index_changes
        .iter()
        .map(|(path, change)| StagedFile {
            path: path.clone(),
            change_type: match change {
                ChangeType::Added => "Added".to_string(),
                ChangeType::Modified => "Modified".to_string(),
                ChangeType::Deleted => "Deleted".to_string(),
            },
        })
        .collect();

    Ok(StatusInfo {
        untracked: status.untracked,
        staged,
    })
}

#[tauri::command]
pub fn add_file(path: String, state: State<AppState>) -> Result<(), String> {
    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_ref()
        .ok_or_else(|| "No repository open".to_string())?;

    let mut cmd = AddCommand {
        root_path: repo.work_tree.path().to_string_lossy().to_string(),
        path: PathBuf::from(path),
    };

    cmd.run().map_err(|e| e.to_string())
}