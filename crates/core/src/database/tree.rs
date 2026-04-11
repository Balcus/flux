use super::object_type::ObjectType;
use crate::{
    database::{blob::Blob, object::Object, tree_entry::TreeEntry},
    utils::{
        self,
        modes::{MODE_DIR, MODE_EXEC, MODE_FILE},
    },
};
use std::os::unix::fs::MetadataExt;
use std::{any::Any, collections::HashMap, fmt, fs, path::Path};

#[derive(Clone)]
pub struct Tree {
    content: Vec<u8>,
}

impl Tree {
    pub fn new(dir: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut entries = Vec::new();
        let dir_iter = fs::read_dir(dir)?;

        for entry in dir_iter {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                if name.starts_with('.') {
                    continue;
                }

                let metadata = fs::metadata(&path)?;
                let name = name.to_string();

                let mode = if metadata.is_dir() {
                    MODE_DIR
                } else if metadata.mode() & 0o111 != 0 {
                    MODE_EXEC
                } else {
                    MODE_FILE
                };

                if metadata.is_file() {
                    let data = fs::read(&path)?;
                    let blob = Blob::from_bytes(data);
                    entries.push(TreeEntry {
                        mode,
                        id: blob.id(),
                        name,
                    });
                } else if metadata.is_dir() {
                    let subtree = Tree::new(&path)?;
                    entries.push(TreeEntry {
                        mode,
                        id: subtree.id(),
                        name,
                    });
                }
            }
        }

        let content = Self::build_content(entries);
        Ok(Self { content })
    }

    pub fn from_index(index: &HashMap<String, String>) -> Self {
        let mut entries = Vec::new();

        for (path_str, id) in index {
            let path = Path::new(path_str);
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path_str)
                .to_string();

            let mode = if let Ok(meta) = fs::metadata(path) {
                if meta.is_dir() {
                    MODE_DIR
                } else if meta.mode() & 0o111 != 0 {
                    MODE_EXEC
                } else {
                    MODE_FILE
                }
            } else {
                MODE_FILE
            };

            entries.push(TreeEntry {
                mode,
                id: id.clone(),
                name,
            });
        }
        let content = Self::build_content(entries);
        Self { content }
    }

    fn build_content(mut entries: Vec<TreeEntry>) -> Vec<u8> {
        entries.sort_by(|a, b| {
            let mut a_sort = a.name.clone();
            let mut b_sort = b.name.clone();
            if a.is_dir() {
                a_sort.push('/');
            }
            if b.is_dir() {
                b_sort.push('/');
            }
            a_sort.cmp(&b_sort)
        });

        let mut tree_content = Vec::new();
        for entry in entries {
            let hash_bytes = hex::decode(&entry.id).expect("Invalid object hash");
            // Modes are already normalized in TreeEntry now
            let entry_header = format!("{:o} {}\0", entry.mode, entry.name);

            tree_content.extend_from_slice(entry_header.as_bytes());
            tree_content.extend_from_slice(&hash_bytes);
        }
        tree_content
    }

    pub fn from_content(content: Vec<u8>) -> Self {
        Self { content }
    }

    pub fn entries(&self) -> Vec<TreeEntry> {
        let mut entries = Vec::new();
        let mut pos = 0;

        while pos < self.content.len() {
            let Some(space_pos) = self.content[pos..].iter().position(|&b| b == b' ') else {
                break;
            };
            let mode_str = String::from_utf8_lossy(&self.content[pos..pos + space_pos]);
            let mode = u32::from_str_radix(&mode_str, 8).unwrap_or(0);
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
            let id = hex::encode(hash_bytes);
            pos += 20;

            entries.push(TreeEntry { mode, id, name });
        }
        entries
    }

    fn to_string_pretty(&self) -> String {
        let mut result = String::new();
        for entry in self.entries() {
            let type_str = if entry.is_dir() { "tree" } else { "blob" };
            result.push_str(&format!(
                "{:06o} {} {} {}\n",
                entry.mode, type_str, entry.id, entry.name
            ));
        }
        result
    }
}

impl Object for Tree {
    fn object_type(&self) -> ObjectType {
        ObjectType::Tree
    }

    fn id(&self) -> String {
        let mut full = format!("tree {}\0", self.content.len()).into_bytes();
        full.extend_from_slice(&self.content);
        utils::hash(&full)
    }

    fn serialize(&self) -> Vec<u8> {
        let mut full = format!("tree {}\0", self.content.len()).into_bytes();
        full.extend_from_slice(&self.content);
        full
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn content(&self) -> Vec<u8> {
        self.content.clone()
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_pretty())
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
        assert!(entries.iter().all(|e| !e.is_dir()));
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

        assert!(!file_entry.is_dir());
        assert!(dir_entry.is_dir());

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
        assert_eq!(entries[0].id, blob.id());
        assert!(!entries[0].is_dir());
    }
}
