use flux_core::internals::repository::Repository;
use flux_core::error::ConfigError;
use serde::Serialize;

#[derive(Serialize)]
pub struct RepositoryInfo {
    pub path: String,
    pub branches: Vec<BranchInfo>,
    pub head: String,
    pub index: Vec<String>,
    pub uncommited: Vec<String>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub origin: Option<String>
}

impl RepositoryInfo {
    pub fn from_repo(repo: &Repository) -> Result<Self, String> {
        let head = repo.refs.head_ref().map_err(|e| e.to_string())?;
        let current = repo.refs.current_branch().map_err(|e| e.to_string())?;
        
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
        
        let user_name = repo.config.get("user_name")
            .map_err(|e: ConfigError| e.to_string())?;
        let user_email = repo.config.get("user_email")
            .map_err(|e: ConfigError| e.to_string())?;
        let origin = repo.config.get("origin")
            .map_err(|e: ConfigError| e.to_string())?;
        
        Ok(Self {
            path: repo.work_tree.path().to_string_lossy().to_string(),
            head,
            branches,
            index: repo.index.map.keys().cloned().collect(),
            uncommited: Vec::new(),
            user_name,
            user_email,
            origin
        })
    }
}

#[derive(Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
}