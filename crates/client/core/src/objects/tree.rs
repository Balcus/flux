use super::object_type::ObjectType;
use crate::{
    objects::{blob::Blob, object::Object}, utils::{self, read_bytes_from_file}
};
use std::{any::Any, collections::HashMap, fmt, fs, path::Path};

pub struct TreeEntry {
    pub mode: String,
    pub hash: String,
    pub name: String,
}

impl TreeEntry {
    pub fn is_dir(&self) -> bool {
        self.mode == "040000"
    }

    pub fn is_file(&self) -> bool {
        self.mode != "040000"
    }
}

pub struct Tree {
    content: Vec<u8>,
}

impl Tree {
    pub fn new(dir: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut entries = Vec::new();
        let dir_iter = fs::read_dir(dir).expect("Could not read directory contents");

        for entry in dir_iter {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                if name.starts_with('.') {
                    continue;
                }

                let metadata = fs::metadata(&path).expect("Could not read file metadata");
                let name = name.to_string();

                if metadata.is_file() {
                    let data = read_bytes_from_file(&path)?;
                    let blob = Blob::from_bytes(data);

                    let hash = blob.id();

                    entries.push(TreeEntry {
                        mode: "100644".to_string(),
                        hash,
                        name,
                    });
                } else if metadata.is_dir() {
                    let subtree = Tree::new(&path)?;
                    let hash = subtree.id();

                    entries.push(TreeEntry {
                        mode: "040000".to_string(),
                        hash,
                        name,
                    });
                }
            }
        }

        let content = Self::build_content(entries);
        Ok(Self { content })
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

impl Object for Tree {
    fn object_type(&self) -> ObjectType {
        ObjectType::Tree
    }

    fn id(&self) -> String {
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn content(&self) -> Vec<u8> {
        self.content.clone()
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn tree_from_directory() -> Result<()> {
        let dir = tempdir()?;
        fs::write(dir.path().join("a.txt"), "file a")?;
        fs::write(dir.path().join("b.txt"), "file b")?;

        let tree = Tree::new(dir.path()).unwrap();
        let entries = tree.entries();

        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.is_file()));
        assert!(entries.iter().any(|e| e.name == "a.txt"));
        assert!(entries.iter().any(|e| e.name == "b.txt"));
        assert_eq!(tree.object_type(), ObjectType::Tree);

        Ok(())
    }

    #[test]
    fn tree_entries_are_sorted() -> Result<()> {
        let dir = tempdir()?;
        fs::write(dir.path().join("z.txt"), "z")?;
        fs::write(dir.path().join("a.txt"), "a")?;
        fs::write(dir.path().join("m.txt"), "m")?;

        let tree = Tree::new(dir.path()).unwrap();
        let entries = tree.entries();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();

        assert_eq!(names, vec!["a.txt", "m.txt", "z.txt"]);

        Ok(())
    }

    #[test]
    fn tree_with_subtree() -> Result<()> {
        let dir = tempdir()?;
        fs::write(dir.path().join("root.txt"), "root")?;
        fs::create_dir(dir.path().join("subdir"))?;
        fs::write(dir.path().join("subdir/child.txt"), "child")?;

        let tree = Tree::new(dir.path()).unwrap();
        let entries = tree.entries();

        let file_entry = entries.iter().find(|e| e.name == "root.txt").unwrap();
        let dir_entry = entries.iter().find(|e| e.name == "subdir").unwrap();

        assert!(file_entry.is_file());
        assert!(dir_entry.is_dir());
        assert_eq!(dir_entry.mode, "040000");

        Ok(())
    }

    #[test]
    fn tree_id_changes_with_content() -> Result<()> {
        let dir1 = tempdir()?;
        let dir2 = tempdir()?;
        fs::write(dir1.path().join("file.txt"), "version one")?;
        fs::write(dir2.path().join("file.txt"), "version two")?;

        let tree1 = Tree::new(dir1.path()).unwrap();
        let tree2 = Tree::new(dir2.path()).unwrap();

        assert_ne!(tree1.id(), tree2.id());

        Ok(())
    }

    #[test]
    fn tree_skips_hidden_files() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let dir = tempdir()?;
        fs::write(dir.path().join("visible.txt"), "visible")?;
        fs::write(dir.path().join(".hidden"), "hidden")?;
        fs::create_dir(dir.path().join(".flux"))?;

        let tree = Tree::new(dir.path())?;
        let entries = tree.entries();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "visible.txt");

        Ok(())
    }

    #[test]
    fn tree_from_index() {
        let mut index = HashMap::new();
        let blob = Blob::new("hello");
        index.insert("src/main.rs".to_string(), blob.id());

        let tree = Tree::from_index(&index);
        let entries = tree.entries();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "main.rs");
        assert_eq!(entries[0].hash, blob.id());
        assert_eq!(entries[0].mode, "100644");
    }
}
