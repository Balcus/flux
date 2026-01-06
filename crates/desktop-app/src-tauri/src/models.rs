use flux_core::repo::repository::Repository;
use serde::Serialize;

#[derive(Serialize)]
pub struct RepositoryInfo {
    pub path: String,
    pub branches: Vec<BranchInfo>,
    pub head: String,
    pub index: Vec<String>,
    pub uncommited: Vec<String>,
}

impl RepositoryInfo {
    pub fn from_repo(repo: &Repository) -> Self {
        Self {
            path: repo.work_tree.path().to_string_lossy().to_string(),
            head: repo.head.clone(),
            branches: repo.branches.iter().map(BranchInfo::from).collect(),
            index: repo.index.map.keys().cloned().collect(),
            uncommited: Vec::new(),
        }
    }
}

#[derive(Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
}

impl From<&flux_core::repo::branch::Branch> for BranchInfo {
    fn from(branch: &flux_core::repo::branch::Branch) -> Self {
        Self {
            name: branch.name.clone(),
            is_current: branch.is_current,
        }
    }
}
