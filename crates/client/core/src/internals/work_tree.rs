use crate::error;
use crate::internals::object_store::ObjectStore;
use crate::objects::blob::Blob;
use crate::objects::commit::Commit;
use crate::objects::object_type::FluxObject;
use crate::objects::tree::{Tree, TreeEntry};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct WorkTree {
    path: PathBuf,
}
// TODO: operations like switch branch currently fail if there is binary data inside the work tree when doing a commit
#[derive(Debug)]
enum TreeNode {
    File(String),
    Dir(HashMap<String, TreeNode>),
}

impl WorkTree {
    pub fn new(project_path: PathBuf) -> Self {
        Self { path: project_path }
    }

    pub fn clear(&self) -> Result<(), error::WorkTreeError> {
        let iter = fs::read_dir(&self.path).map_err(|e| error::IoError::Read {
            path: self.path.clone(),
            source: e,
        })?;

        for entry in iter {
            let entry = entry.map_err(|e| error::IoError::Read {
                path: self.path.clone(),
                source: e,
            })?;

            let path = entry.path();
            if path.file_name().and_then(|n| n.to_str()) == Some(".flux") {
                continue;
            }

            let ft = entry.file_type().map_err(|e| error::IoError::Read {
                path: path.clone(),
                source: e,
            })?;

            if ft.is_file() || ft.is_symlink() {
                fs::remove_file(&path).map_err(|e| error::IoError::Delete {
                    path: path.clone(),
                    source: e,
                })?;
            } else if ft.is_dir() {
                fs::remove_dir_all(&path).map_err(|e| error::IoError::Delete {
                    path: path.clone(),
                    source: e,
                })?;
            }
        }

        Ok(())
    }

    pub fn restore_from_commit(
        &self,
        commit_hash: &str,
        object_store: &ObjectStore,
    ) -> Result<(), error::WorkTreeError> {
        let commit_obj = object_store.retrieve_object(commit_hash)?;
        let commit = commit_obj
            .as_any()
            .downcast_ref::<Commit>()
            .ok_or(error::WorkTreeError::Downcast { expected: "commit" })?;
        let tree_hash = &commit.tree_hash;
        self.restore_tree(tree_hash, &self.path, object_store)?;

        Ok(())
    }

    fn restore_tree(
        &self,
        tree_hash: &str,
        target_dir: &Path,
        object_store: &ObjectStore,
    ) -> Result<(), error::WorkTreeError> {
        let tree_obj = object_store.retrieve_object(tree_hash)?;

        let tree = tree_obj
            .as_any()
            .downcast_ref::<Tree>()
            .ok_or(error::WorkTreeError::Downcast { expected: "tree" })?;

        let entries = tree.entries();

        for entry in entries {
            let target_path = target_dir.join(&entry.name);

            if entry.mode.starts_with("040") {
                fs::create_dir_all(&target_path).map_err(|e| error::IoError::Create {
                    path: target_path.clone(),
                    source: e,
                })?;
                self.restore_tree(&entry.hash, &target_path, object_store)?;
            } else {
                let blob_obj = object_store.retrieve_object(&entry.hash)?;

                let blob = blob_obj
                    .as_any()
                    .downcast_ref::<Blob>()
                    .ok_or(error::WorkTreeError::Downcast { expected: "blob" })?;

                let blob_content = blob.to_string();
                fs::write(&target_path, blob_content.as_bytes()).map_err(|e| {
                    error::IoError::Write {
                        path: target_path.clone(),
                        source: e,
                    }
                })?;
            }
        }

        Ok(())
    }

    pub fn build_tree_from_index(
        &self,
        index: &HashMap<String, String>,
        object_store: &ObjectStore,
    ) -> Result<String, error::WorkTreeError> {
        let root = self.build_tree_structure(index);
        let hash = self.create_tree_object(&root, object_store)?;
        Ok(hash)
    }

    fn build_tree_structure(&self, index: &HashMap<String, String>) -> TreeNode {
        let mut root = TreeNode::Dir(HashMap::new());

        for (path, hash) in index {
            let parts: Vec<&str> = path.split('/').collect();
            let mut current = &mut root;

            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    if let TreeNode::Dir(map) = current {
                        map.insert(part.to_string(), TreeNode::File(hash.clone()));
                    }
                } else if let TreeNode::Dir(map) = current {
                    current = map
                        .entry(part.to_string())
                        .or_insert_with(|| TreeNode::Dir(HashMap::new()));
                }
            }
        }

        root
    }

    fn create_tree_object(
        &self,
        node: &TreeNode,
        object_store: &ObjectStore,
    ) -> Result<String, error::WorkTreeError> {
        match node {
            TreeNode::File(hash) => Ok(hash.clone()),
            TreeNode::Dir(map) => {
                let mut entries = Vec::new();
                for (name, child) in map {
                    match child {
                        TreeNode::File(hash) => {
                            entries.push(TreeEntry {
                                mode: "100644".to_string(),
                                hash: hash.clone(),
                                name: name.clone(),
                            });
                        }
                        TreeNode::Dir(_) => {
                            let subtree_hash = self.create_tree_object(child, object_store)?;
                            entries.push(TreeEntry {
                                mode: "040000".to_string(),
                                hash: subtree_hash,
                                name: name.clone(),
                            });
                        }
                    }
                }

                entries.sort_by(|a, b| {
                    let a_name = if a.mode == "040000" {
                        format!("{}/", a.name)
                    } else {
                        a.name.clone()
                    };
                    let b_name = if b.mode == "040000" {
                        format!("{}/", b.name)
                    } else {
                        b.name.clone()
                    };
                    a_name.cmp(&b_name)
                });

                let mut tree_content = Vec::new();
                for entry in entries {
                    let hash_bytes = hex::decode(&entry.hash).map_err(|e| {
                        error::WorkTreeError::InvalidHash {
                            hash: entry.hash,
                            source: e,
                        }
                    })?;
                    let entry_header = format!("{} {}\0", entry.mode, entry.name);
                    tree_content.extend_from_slice(entry_header.as_bytes());
                    tree_content.extend_from_slice(&hash_bytes);
                }

                let tree = Tree::from_content(tree_content);
                object_store.store(&tree)?;

                Ok(tree.hash())
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
