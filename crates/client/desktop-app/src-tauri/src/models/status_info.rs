#[derive(serde::Serialize)]
pub struct StatusInfo {
    pub untracked: Vec<String>,
    pub staged: Vec<StagedFile>,
}

#[derive(serde::Serialize)]
pub struct StagedFile {
    pub path: String,
    pub change_type: String,
}