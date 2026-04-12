use crate::models::stage_file::StagedFile;
use serde::Serialize;

#[derive(Serialize)]
pub struct StatusInfo {
    pub staged: Vec<StagedFile>,
    pub unstaged: Vec<StagedFile>,
    pub untracked: Vec<String>,
}
