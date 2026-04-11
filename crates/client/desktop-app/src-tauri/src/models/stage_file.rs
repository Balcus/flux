use serde::Serialize;

#[derive(Serialize, PartialEq, PartialOrd, Eq, Ord)]
pub struct StagedFile {
    pub path: String,
    pub change_type: String,
}
