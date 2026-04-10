use serde::Serialize;

#[derive(Serialize)]
pub struct StagedFile {
    pub path: String,
    pub change_type: String,
}

#[derive(Serialize)]
pub struct StatusInfo {
    pub staged: Vec<StagedFile>,
    pub unstaged: Vec<StagedFile>,
    pub untracked: Vec<String>,
}