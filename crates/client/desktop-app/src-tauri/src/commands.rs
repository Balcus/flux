use crate::models::{
    app_state::AppState, branch_info::BranchInfo, commit_graph_info::CommitGraphInfo,
    graph_edge::GraphEdge, graph_node::GraphNode, repository_info::RepositoryInfo,
};
use flux_core::{
    commands::{
        add::AddCommand, command::Command, commit::CommitCommand, reset::ResetCommand,
        restore::RestoreCommand, rm::RmCommand,
    },
    database::database::Database,
    error::{ConfigError, RefsError},
    internals::repository::Repository,
    traversal::commit_walker::CommitWalker,
};
use std::path::PathBuf;
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
pub fn get_graph(state: State<AppState>) -> Result<CommitGraphInfo, String> {
    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_ref()
        .ok_or_else(|| "No repository open".to_string())?;

    let db = Database::open(repo.flux_dir.clone());
    let walker = CommitWalker::new(&db);
    let graph = walker.walk(&repo.refs).map_err(|e| e.to_string())?;

    Ok(CommitGraphInfo {
        nodes: graph
            .nodes
            .iter()
            .map(|n| GraphNode {
                id: n.id.clone(),
                short_id: n.short_id.clone(),
                message: n.message.clone(),
                author: n.author.clone(),
                branches: n.branches.clone(),
                parents: n.parents.clone(),
                is_merge: n.is_merge,
            })
            .collect(),
        edges: graph
            .edges
            .iter()
            .map(|e| GraphEdge {
                id: e.id.clone(),
                source: e.source.clone(),
                target: e.target.clone(),
            })
            .collect(),
    })
}

#[tauri::command]
pub fn add(path: String, state: State<AppState>) -> Result<(), String> {
    let mut repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_mut()
        .ok_or_else(|| "No repository open".to_string())?;

    let mut cmd = AddCommand {
        repo,
        path: PathBuf::from(path),
    };

    cmd.run().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rm(path: String, state: State<AppState>) -> Result<(), String> {
    let mut repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_mut()
        .ok_or_else(|| "No repository open".to_string())?;
    let mut cmd = RmCommand { repo, path };

    cmd.run().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_soft(path: String, state: State<AppState>) -> Result<(), String> {
    let mut repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_mut()
        .ok_or_else(|| "No repository open".to_string())?;
    let mut cmd = ResetCommand::new(repo, path, false);
    cmd.run().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_hard(path: String, state: State<AppState>) -> Result<(), String> {
    let mut repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_mut()
        .ok_or_else(|| "No repository open".to_string())?;
    let mut cmd = ResetCommand::new(repo, path, true);
    cmd.run().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore(path: String, state: State<AppState>) -> Result<(), String> {
    let mut repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_mut()
        .ok_or_else(|| "No repository open".to_string())?;
    let mut cmd = RestoreCommand::new(repo, path);
    cmd.run().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_diff(path: String, state: State<AppState>) -> Result<String, String> {
    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock
        .as_ref()
        .ok_or_else(|| "No repository open".to_string())?;

    let workspace_path = repo.work_tree.path().join(&path);
    let workspace_data = std::fs::read_to_string(&workspace_path).unwrap_or_default();

    let mut index = flux_core::dircache::index::Index::new(repo.flux_dir.join("index"));
    index.load().map_err(|e| e.to_string())?;

    let index_data = match index.entries.get(&(path.clone(), 0)) {
        Some(entry) => {
            let db = Database::open(repo.flux_dir.clone());
            let blob = db
                .read_object(&hex::encode(entry.id))
                .map_err(|e| e.to_string())?;
            String::from_utf8(blob.content()).unwrap_or_default()
        }
        None => String::new(),
    };

    let hunks = flux_core::diff::diff_impl::Mayers::diff_hunks(&index_data, &workspace_data);

    if hunks.is_empty() {
        return Ok(String::new());
    }

    let mut out = String::new();
    for hunk in &hunks {
        out.push_str(&hunk.header());
        out.push('\n');
        for edit in &hunk.edits {
            use flux_core::diff::edit::EditType;
            let line = edit.a_line.as_ref().or(edit.b_line.as_ref()).unwrap();
            let prefix = match edit.edit_type {
                EditType::Insertion => "+",
                EditType::Deletion => "-",
                EditType::Equal => " ",
            };
            out.push_str(&format!("{} {}\n", prefix, line.text));
        }
    }

    Ok(out)
}

#[tauri::command]
pub fn commit(message: String, state: State<AppState>) -> Result<(), String> {
    let mut repo_lock = state.repository.lock().unwrap();
    let mut repo = repo_lock
        .as_mut()
        .ok_or_else(|| "No repository open".to_string())?;

    CommitCommand::new(&mut repo, message)
        .map_err(|e| e.to_string())?
        .run()
        .map_err(|e| e.to_string())
}
