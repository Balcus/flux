use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author: String,
    pub parent: Option<String>,
    pub branch: String,
}
