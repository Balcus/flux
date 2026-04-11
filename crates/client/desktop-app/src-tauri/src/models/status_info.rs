use serde::Serialize;
use crate::models::stage_file::StagedFile;

#[derive(Serialize)]
pub struct StatusInfo {
    pub staged: Vec<StagedFile>,
    pub unstaged: Vec<StagedFile>,
    pub untracked: Vec<String>,
}
