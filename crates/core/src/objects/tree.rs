use crate::{
    objects::{blob::Blob, object_type::FluxObject},
    utils,
};

use super::object_type::ObjectType;
use std::{any::Any, collections::HashMap, fs, path::Path};

pub struct TreeEntry {
    pub mode: String,
    pub hash: String,
    pub name: String,
}

pub struct Tree {
    content: Vec<u8>,
}

impl Tree {
    pub fn new(dir: &Path) -> Self {
        let mut entries = Vec::new();
        let dir_iter = fs::read_dir(dir).expect("Could not read directory contents");

        for entry in dir_iter {
            let entry = entry.expect("Could not read directory entry");
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                if name.starts_with('.') {
                    continue;
                }

                let metadata = fs::metadata(&path).expect("Could not read file metadata");
                let name = name.to_string();

                if metadata.is_file() {
                    let blob = Blob::new(&path);
                    let hash = blob.hash();

                    entries.push(TreeEntry {
                        mode: "100644".to_string(),
                        hash,
                        name,
                    });
                } else if metadata.is_dir() {
                    let subtree = Tree::new(&path);
                    let hash = subtree.hash();

                    entries.push(TreeEntry {
                        mode: "040000".to_string(),
                        hash,
                        name,
                    });
                }
            }
        }

        let content = Self::build_content(entries);
        Self { content }
    }

    pub fn from_content(content: Vec<u8>) -> Self {
        Self { content }
    }

    pub fn from_index(index: &HashMap<String, String>) -> Self {
        let mut entries = Vec::new();

        for (path, hash) in index {
            let name = Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .expect("Invalid filename in index")
                .to_string();

            entries.push(TreeEntry {
                mode: "100644".to_string(),
                hash: hash.clone(),
                name,
            });
        }
        let content = Self::build_content(entries);

        Self { content }
    }

    fn build_content(mut entries: Vec<TreeEntry>) -> Vec<u8> {
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
            let hash_bytes = hex::decode(&entry.hash).expect("Invalid object hash");
            let entry_header = format!("{} {}\0", entry.mode, entry.name);

            tree_content.extend_from_slice(entry_header.as_bytes());
            tree_content.extend_from_slice(&hash_bytes);
        }

        tree_content
    }

    fn to_string(&self) -> String {
        let mut result = String::new();
        let mut pos = 0;

        while pos < self.content.len() {
            if let Some(space_pos) = self.content[pos..].iter().position(|&b| b == b' ') {
                let mode = String::from_utf8_lossy(&self.content[pos..pos + space_pos]);
                pos += space_pos + 1;

                if let Some(null_pos) = self.content[pos..].iter().position(|&b| b == 0) {
                    let name = String::from_utf8_lossy(&self.content[pos..pos + null_pos]);
                    pos += null_pos + 1;

                    let hash_bytes = &self.content[pos..pos + 20];
                    let hash: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();
                    pos += 20;

                    let entry_type = if mode.starts_with("040") {
                        "tree"
                    } else {
                        "blob"
                    };
                    result.push_str(&format!("{} {} {} {}\n", mode, entry_type, hash, name));
                }
            }
        }

        result
    }

    pub fn entries(&self) -> Vec<TreeEntry> {
        let mut entries = Vec::new();
        let mut pos = 0;

        while pos < self.content.len() {
            let Some(space_pos) = self.content[pos..].iter().position(|&b| b == b' ') else {
                break;
            };
            let mode = String::from_utf8_lossy(&self.content[pos..pos + space_pos]).to_string();
            pos += space_pos + 1;

            let Some(null_pos) = self.content[pos..].iter().position(|&b| b == 0) else {
                break;
            };
            let name = String::from_utf8_lossy(&self.content[pos..pos + null_pos]).to_string();
            pos += null_pos + 1;

            if pos + 20 > self.content.len() {
                break;
            }

            let hash_bytes = &self.content[pos..pos + 20];
            let hash: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();
            pos += 20;

            entries.push(TreeEntry { mode, hash, name });
        }

        entries
    }
}

impl FluxObject for Tree {
    fn object_type(&self) -> ObjectType {
        ObjectType::Tree
    }

    fn hash(&self) -> String {
        let header = format!("tree {}\0", self.content.len());
        let mut full = Vec::new();
        full.extend_from_slice(header.as_bytes());
        full.extend_from_slice(&self.content);
        utils::hash(&full)
    }

    fn serialize(&self) -> Vec<u8> {
        let header = format!("tree {}\0", self.content.len());
        let mut full = Vec::new();
        full.extend_from_slice(header.as_bytes());
        full.extend_from_slice(&self.content);
        utils::compress(&full)
    }

    fn print(&self) {
        print!("{}", self.to_string());
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn content(&self) -> Vec<u8> {
        self.content.clone()
    }
}
