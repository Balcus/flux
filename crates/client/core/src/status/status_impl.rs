use crate::internals::repository::Repository;
use crate::objects::blob::Blob;
use crate::objects::object_type::FluxObject;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::anyhow;

pub struct Status {
    pub index_changes: HashMap<String, ChangeType>,
    pub workspace_changes: HashMap<String, ChangeType>,
    pub untracked: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}

impl Status {
    pub fn new(repo: &Repository) -> Result<Self> {
        let index = &repo.index.map;
        let head_snapshot = Self::get_head_snapshot(repo)?;
        let index_changes = Self::compare_head_to_index(&head_snapshot, index);
        let workspace_changes = Self::compare_index_to_workspace(repo, index)?;
        let untracked = Self::find_untracked_files(repo, index)?;

        Ok(Self {
            index_changes,
            workspace_changes,
            untracked,
        })
    }

    fn get_head_snapshot(repo: &Repository) -> Result<HashMap<String, String>> {
        match repo.refs.head_commit() {
            Ok(hash) if !hash.is_empty() => Ok(repo.object_store.commit_to_map(hash)?),
            _ => Ok(HashMap::new()),
        }
    }

    fn compare_head_to_index(
        head_snapshot: &HashMap<String, String>,
        index: &HashMap<String, String>,
    ) -> HashMap<String, ChangeType> {
        let mut changes = HashMap::new();

        for (path, index_hash) in index {
            match head_snapshot.get(path) {
                Some(head_hash) if head_hash != index_hash => {
                    changes.insert(path.clone(), ChangeType::Modified);
                }
                None => {
                    changes.insert(path.clone(), ChangeType::Added);
                }
                _ => {}
            }
        }

        for (path, _) in head_snapshot {
            if !index.contains_key(path) {
                changes.insert(path.clone(), ChangeType::Deleted);
            }
        }

        changes
    }

    fn compare_index_to_workspace(
        repo: &Repository,
        index: &HashMap<String, String>,
    ) -> Result<HashMap<String, ChangeType>> {
        let mut changes = HashMap::new();

        for (rel_path, index_hash) in index {
            let full_path = repo.work_tree.path().join(rel_path);

            if !full_path.exists() {
                changes.insert(rel_path.clone(), ChangeType::Deleted);
            } else if full_path.is_file() {
                let current_blob = Blob::new(&full_path);
                if &current_blob.hash() != index_hash {
                    changes.insert(rel_path.clone(), ChangeType::Modified);
                }
            }
        }

        Ok(changes)
    }

    fn find_untracked_files(
        repo: &Repository,
        index: &HashMap<String, String>,
    ) -> Result<Vec<String>> {
        let mut untracked = Vec::new();
        Self::scan_directory(
            repo.work_tree.path(),
            repo.work_tree.path(),
            index,
            &mut untracked,
        )?;
        untracked.sort();
        Ok(untracked)
    }

    fn scan_directory(
        root: &Path,
        current: &Path,
        index: &HashMap<String, String>,
        untracked: &mut Vec<String>,
    ) -> Result<()> {
        if current.ends_with(".flux") {
            return Ok(());
        }

        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();

            if path.ends_with(".flux") {
                continue;
            }

            let rel_path = path.strip_prefix(root)?;
            let rel_str = rel_path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid UTF-8 in path"))?;

            if path.is_file() {
                if !index.contains_key(rel_str) {
                    untracked.push(rel_str.to_string());
                }
            } else if path.is_dir() {
                Self::scan_directory(root, &path, index, untracked)?;
            }
        }

        Ok(())
    }

    pub fn is_clean(&self) -> bool {
        self.index_changes.is_empty()
            && self.workspace_changes.is_empty()
            && self.untracked.is_empty()
    }

    pub fn has_staged_changes(&self) -> bool {
        !self.index_changes.is_empty()
    }

    pub fn has_unstaged_changes(&self) -> bool {
        !self.workspace_changes.is_empty()
    }

    pub fn has_untracked_files(&self) -> bool {
        !self.untracked.is_empty()
    }
}
