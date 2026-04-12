use crate::utils::modes::{S_IFDIR, S_IFMT};

pub struct TreeEntry {
    pub id: String,
    pub mode: u32,
    pub name: String,
}

impl TreeEntry {
    pub fn is_dir(&self) -> bool {
        (self.mode & S_IFMT) == S_IFDIR
    }
}
