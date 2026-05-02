use crate::database::blob::Blob;
use crate::database::commit::Commit;
use crate::database::database::Database;
use crate::database::object::Object;
use crate::database::tree::Tree;
use crate::database::tree_entry::TreeEntry;
use crate::dircache::index::Index;
use anyhow::Context;
use std::collections::HashMap;
use std::fs::{self, Metadata};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct WorkTree {
    path: PathBuf,
    ignore: Vec<String>,
}

#[derive(Debug)]
enum TreeNode {
    File(String),
    Dir(HashMap<String, TreeNode>),
}

impl WorkTree {
    pub fn new(project_path: PathBuf) -> Self {
        let mut ignore = vec![".".to_string(), "..".to_string(), ".flux".to_string()];

        for ignore_file in &[".fluxignore", ".gitignore", ".ignore"] {
            let path = project_path.join(ignore_file);
            if let Ok(contents) = std::fs::read_to_string(&path) {
                for line in contents.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        ignore.push(line.to_string());
                    }
                }
            }
        }

        Self {
            path: project_path,
            ignore,
        }
    }

    fn glob_match(pattern: &str, name: &str) -> bool {
        let parts: Vec<&str> = pattern.splitn(2, '*').collect();
        match parts.as_slice() {
            [prefix, suffix] => name.starts_with(prefix) && name.ends_with(suffix),
            _ => pattern == name,
        }
    }

    pub fn is_ignored(&self, filename: &str, is_dir: bool) -> bool {
        for pattern in &self.ignore {
            let dir_only = pattern.ends_with('/');
            let clean = pattern.trim_end_matches('/').trim_start_matches('/');

            if dir_only && !is_dir {
                continue;
            }

            if clean.contains('*') {
                if Self::glob_match(clean, filename) {
                    return true;
                }
            } else if clean == filename {
                return true;
            }
        }
        false
    }

    pub fn list_files(&self, path: Option<&Path>) -> anyhow::Result<Vec<PathBuf>> {
        let current_path = path.unwrap_or(&self.path);

        let relative_path = current_path
            .strip_prefix(&self.path)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| current_path.to_path_buf());

        if current_path.is_dir() {
            let local_ignore = self.load_local_ignores(current_path);
            let mut all_files = Vec::new();
            for entry in fs::read_dir(current_path)? {
                let entry = entry?;
                let filename = entry.file_name();
                let filename_str = filename.to_str().unwrap_or("");
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

                if !self.is_ignored(filename_str, is_dir)
                    && !Self::is_locally_ignored(&local_ignore, filename_str, is_dir)
                {
                    let child_path = entry.path();
                    let mut sub_files = self.list_files(Some(&child_path))?;
                    all_files.append(&mut sub_files);
                }
            }
            Ok(all_files)
        } else if current_path.exists() {
            Ok(vec![relative_path])
        } else {
            anyhow::bail!("Path '{}' did not match any file.", relative_path.display())
        }
    }

    pub fn list_dir(&self, dirname: Option<&Path>) -> anyhow::Result<HashMap<PathBuf, Metadata>> {
        let path = self.path.join(dirname.unwrap_or(Path::new("")));
        if !path.is_dir() {
            anyhow::bail!("Path '{}' is not a directory.", path.display());
        }

        let mut stats = HashMap::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let filename = entry.file_name();
            let filename_str = filename.to_str().unwrap_or("");
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

            if !self.is_ignored(filename_str, is_dir) {
                let relative = entry
                    .path()
                    .strip_prefix(&self.path)
                    .map(|e| e.to_path_buf())?;
                let stat = fs::metadata(entry.path())?;
                stats.insert(relative, stat);
            }
        }

        Ok(stats)
    }

    pub fn read_file(&self, path: &Path) -> anyhow::Result<Vec<u8>> {
        let full_path = self.path.join(path);
        if !full_path.is_file() {
            anyhow::bail!("Path '{}' is not a file", &full_path.display());
        }

        let data = fs::read(full_path)?;
        Ok(data)
    }

    pub fn stat_file(&self, path: &Path) -> anyhow::Result<Option<Metadata>> {
        let full_path = self.path.join(path);
        match full_path.metadata() {
            Ok(metadata) => Ok(Some(metadata)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                anyhow::bail!("stat('{}'): Permission denied", path.display())
            }
            Err(e) => {
                Err(e).with_context(|| format!("stat('{}'): Unexpected error", path.display()))
            }
        }
    }

    pub fn write_file(
        &self,
        path: &Path,
        data: &[u8],
        mode: Option<u32>,
        mkdir: bool,
    ) -> anyhow::Result<()> {
        let full_path = self.path.join(path);

        if mkdir && let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&full_path, data)?;

        if let Some(m) = mode {
            let mut perms = fs::metadata(&full_path)?.permissions();
            perms.set_mode(m);
            fs::set_permissions(&full_path, perms)?;
        }

        Ok(())
    }

    pub fn remove(&self, path: &Path) -> anyhow::Result<()> {
        let full_path = self.path.join(path);

        if full_path.is_dir() {
            fs::remove_dir_all(&full_path)?;
        } else if full_path.exists() {
            fs::remove_file(&full_path)?;
        } else {
            return Ok(());
        }

        let mut parent = full_path.parent();
        while let Some(p) = parent {
            if p == self.path || fs::remove_dir(p).is_err() {
                break;
            }
            parent = p.parent();
        }

        Ok(())
    }

    pub fn remove_dir(&self, path: &Path) {
        let full_path = self.path.join(path);
        if !full_path.is_dir() {
            return;
        }
        let _ = fs::remove_dir(&full_path);
    }

    pub fn make_dir(&self, dirname: &Path) -> anyhow::Result<()> {
        let full_path = self.path.join(dirname);
        let stat = self.stat_file(dirname)?;

        match stat {
            Some(s) if s.is_dir() => Ok(()),
            Some(_) => {
                fs::remove_file(&full_path)?;
                fs::create_dir(&full_path).map_err(Into::into)
            }
            None => fs::create_dir(&full_path).map_err(Into::into),
        }
    }

    pub fn clear(&self) -> anyhow::Result<()> {
        let iter = fs::read_dir(&self.path)
            .with_context(|| format!("Failed to read '{}'.", self.path.display()))?;

        for entry in iter {
            let entry =
                entry.with_context(|| format!("Failed to read '{}'.", self.path.display()))?;

            let path = entry.path();
            if path.file_name().and_then(|n| n.to_str()) == Some(".flux") {
                continue;
            }

            let ft = entry
                .file_type()
                .with_context(|| format!("Failed to read '{}'.", path.display()))?;

            if ft.is_file() || ft.is_symlink() {
                fs::remove_file(&path)
                    .with_context(|| format!("Failed to delete '{}'.", path.display()))?;
            } else if ft.is_dir() {
                fs::remove_dir_all(&path)
                    .with_context(|| format!("Failed to delete '{}'.", path.display()))?;
            }
        }

        Ok(())
    }

    pub fn restore_from_commit(&self, commit_hash: &str) -> anyhow::Result<()> {
        let db = Database::open(self.path.join(".flux"));
        let commit_obj = db.read_object(commit_hash).unwrap();
        let commit = commit_obj
            .as_any()
            .downcast_ref::<Commit>()
            .context("Object downcast error, expected type: 'commit'.")?;
        let tree_hash = &commit.tree_hash;
        self.restore_tree(tree_hash, &self.path, &db)?;
        Ok(())
    }

    fn restore_tree(
        &self,
        tree_hash: &str,
        target_dir: &Path,
        db: &Database,
    ) -> anyhow::Result<()> {
        let tree_obj = db.read_object(tree_hash).unwrap();
        let tree = tree_obj
            .as_any()
            .downcast_ref::<Tree>()
            .context("Object downcast error, expected type: 'tree'.")?;

        for entry in tree.entries() {
            let target_path = target_dir.join(&entry.name);
            if entry.is_dir() {
                fs::create_dir_all(&target_path)
                    .with_context(|| format!("Failed to create '{}'.", target_path.display()))?;
                self.restore_tree(&entry.id, &target_path, db)?;
            } else {
                let blob_obj = db.read_object(&entry.id).unwrap();
                let blob = blob_obj
                    .as_any()
                    .downcast_ref::<Blob>()
                    .context("Object downcast error, expected type: 'blob'.")?;
                fs::write(&target_path, blob.as_string().as_bytes())
                    .with_context(|| format!("Failed to write '{}'.", target_path.display()))?;
            }
        }

        Ok(())
    }

    pub fn build_tree_from_index(&self, index: &Index, db: &Database) -> anyhow::Result<String> {
        let flat: HashMap<String, String> = index
            .entries
            .iter()
            .filter(|((_, stage), _)| *stage == 0)
            .map(|((path, _), entry)| (path.clone(), hex::encode(entry.id)))
            .collect();

        let root = self.build_tree_structure(&flat);
        let hash = self.create_tree_object(&root, db)?;
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

    fn create_tree_object(&self, node: &TreeNode, db: &Database) -> anyhow::Result<String> {
        match node {
            TreeNode::File(id) => Ok(id.clone()),
            TreeNode::Dir(map) => {
                let mut entries = Vec::new();
                for (name, child) in map {
                    match child {
                        TreeNode::File(id) => {
                            entries.push(TreeEntry {
                                mode: 0o100644,
                                id: id.clone(),
                                name: name.clone(),
                            });
                        }
                        TreeNode::Dir(_) => {
                            let subtree_id = self.create_tree_object(child, db)?;
                            entries.push(TreeEntry {
                                mode: 0o040000,
                                id: subtree_id,
                                name: name.clone(),
                            });
                        }
                    }
                }

                entries.sort_by(|a, b| {
                    let a_name = if a.mode == 0o040000 {
                        format!("{}/", a.name)
                    } else {
                        a.name.clone()
                    };
                    let b_name = if b.mode == 0o040000 {
                        format!("{}/", b.name)
                    } else {
                        b.name.clone()
                    };
                    a_name.cmp(&b_name)
                });

                let mut tree_content = Vec::new();
                for entry in entries {
                    let hash_bytes = hex::decode(&entry.id)
                        .map_err(|e| anyhow::anyhow!("Invalid hash {}: {}", entry.id, e))?;
                    let entry_header = format!("{:o} {}\0", entry.mode, entry.name);
                    tree_content.extend_from_slice(entry_header.as_bytes());
                    tree_content.extend_from_slice(&hash_bytes);
                }

                let tree = Tree::from_content(tree_content);
                db.store(Box::new(tree.clone()))?;
                Ok(tree.id())
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load_local_ignores(&self, dir: &Path) -> Vec<String> {
        let mut local = Vec::new();
        for ignore_file in &[".fluxignore", ".gitignore", ".ignore"] {
            let path = dir.join(ignore_file);
            if path == self.path.join(ignore_file) {
                continue; // already loaded at root
            }
            if let Ok(contents) = std::fs::read_to_string(&path) {
                for line in contents.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        local.push(line.to_string());
                    }
                }
            }
        }
        local
    }

    pub fn is_locally_ignored(patterns: &[String], filename: &str, is_dir: bool) -> bool {
        for pattern in patterns {
            let dir_only = pattern.ends_with('/');
            let clean = pattern.trim_end_matches('/').trim_start_matches('/');

            if dir_only && !is_dir {
                continue;
            }

            if clean.contains('*') {
                if Self::glob_match(clean, filename) {
                    return true;
                }
            } else if clean == filename {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn list_files_reccursive() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let repo_path = tmp.path();
        let work_tree = WorkTree::new(repo_path.to_path_buf());

        fs::write(repo_path.join("file1.txt"), "hello")?;
        fs::create_dir(repo_path.join(".flux"))?;
        fs::write(repo_path.join(".flux").join("config"), "secret")?;
        fs::create_dir(repo_path.join("src"))?;
        fs::write(repo_path.join("src").join("main.rs"), "fn main() {}")?;
        fs::write(repo_path.join("src").join("utils.rs"), "pub fn help() {}")?;

        let mut files = work_tree.list_files(None)?;
        files.sort();

        let expected = vec![
            PathBuf::from("file1.txt"),
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/utils.rs"),
        ];

        assert_eq!(files.len(), 3);
        assert_eq!(files, expected);
        assert!(!files.contains(&PathBuf::from(".flux/config")));

        Ok(())
    }

    #[test]
    fn list_dir_with_metadata() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let repo_path = tmp.path();
        let work_tree = WorkTree::new(repo_path.to_path_buf());

        fs::write(repo_path.join("README.md"), "1234567890")?;
        fs::create_dir(repo_path.join("data"))?;
        fs::write(
            repo_path.join("data").join("stats.csv"),
            "01234567890123456789",
        )?;
        fs::write(
            repo_path.join("data").join("notes.txt"),
            "012345678901234567890123456789",
        )?;

        let stats = work_tree.list_dir(Some(Path::new("data")))?;

        assert_eq!(stats.len(), 2);

        let csv_path = PathBuf::from("data/stats.csv");
        let notes_path = PathBuf::from("data/notes.txt");

        assert!(stats.contains_key(&csv_path));
        assert!(stats.contains_key(&notes_path));
        assert_eq!(stats.get(&csv_path).unwrap().len(), 20);
        assert_eq!(stats.get(&notes_path).unwrap().len(), 30);
        assert!(!stats.contains_key(&PathBuf::from("README.md")));

        Ok(())
    }

    #[test]
    fn read_file() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let repo_path = tmp.path();
        let work_tree = WorkTree::new(repo_path.to_path_buf());

        let content = "Hello World!";
        fs::write(repo_path.join("file.txt"), content)?;
        let data = work_tree.read_file(Path::new("file.txt"))?;

        assert_eq!(data, content.as_bytes());
        Ok(())
    }

    #[test]
    fn read_missing_file() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let work_tree = WorkTree::new(tmp.path().to_path_buf());

        let result = work_tree.read_file(Path::new("ghost.txt"));

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not a file"));

        Ok(())
    }

    #[test]
    fn read_dir_instead_of_file() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let repo_path = tmp.path();
        let work_tree = WorkTree::new(repo_path.to_path_buf());

        fs::create_dir(repo_path.join("dir"))?;

        let result = work_tree.read_file(Path::new("dir"));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("is not a file"));

        Ok(())
    }

    #[test]
    fn read_binary_file() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let repo_path = tmp.path();
        let work_tree = WorkTree::new(repo_path.to_path_buf());

        let data_bin = vec![0, 159, 146, 150];
        fs::write(repo_path.join("test.bin"), &data_bin)?;

        let data = work_tree.read_file(Path::new("test.bin"))?;

        assert_eq!(data, data_bin);
        Ok(())
    }

    #[test]
    fn stat_file() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let work_tree = WorkTree::new(tmp.path().to_path_buf());
        let file_path = Path::new("test.txt");

        let meta = work_tree.stat_file(file_path)?;
        assert!(meta.is_none());

        fs::write(tmp.path().join(file_path), "data")?;
        let meta = work_tree.stat_file(file_path)?;
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().len(), 4);

        Ok(())
    }

    #[test]
    fn write_file_with_mkdir() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let work_tree = WorkTree::new(tmp.path().to_path_buf());
        let nested_path = Path::new("a/b/c/file.txt");
        let content = b"nested content";

        let result = work_tree.write_file(nested_path, content, None, false);
        assert!(result.is_err());

        work_tree.write_file(nested_path, content, Some(0o755), true)?;

        let actual_content = fs::read(tmp.path().join(nested_path))?;
        assert_eq!(actual_content, content);

        let meta = fs::metadata(tmp.path().join(nested_path))?;
        assert_eq!(meta.permissions().mode() & 0o777, 0o755);

        Ok(())
    }

    #[test]
    fn remove_with_parent_cleanup() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let work_tree = WorkTree::new(tmp.path().to_path_buf());
        let deep_file = Path::new("deep/dir/structure/file.txt");

        work_tree.write_file(deep_file, b"data", None, true)?;
        assert!(tmp.path().join("deep/dir/structure").exists());

        work_tree.remove(deep_file)?;

        assert!(!tmp.path().join("deep").exists());
        assert!(tmp.path().exists());

        Ok(())
    }

    #[test]
    fn remove_dir_only_if_empty() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let work_tree = WorkTree::new(tmp.path().to_path_buf());
        let dir_path = Path::new("my_dir");

        fs::create_dir(tmp.path().join(dir_path))?;
        fs::write(tmp.path().join(dir_path).join("keep_me.txt"), "data")?;

        work_tree.remove_dir(dir_path);
        assert!(tmp.path().join(dir_path).exists());

        fs::remove_file(tmp.path().join(dir_path).join("keep_me.txt"))?;
        work_tree.remove_dir(dir_path);
        assert!(!tmp.path().join(dir_path).exists());

        Ok(())
    }

    #[test]
    fn make_dir_conflicts() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let work_tree = WorkTree::new(tmp.path().to_path_buf());
        let path = Path::new("clash");

        work_tree.make_dir(path)?;
        assert!(tmp.path().join(path).is_dir());

        fs::remove_dir(tmp.path().join(path))?;
        fs::write(tmp.path().join(path), "I am a file")?;
        assert!(tmp.path().join(path).is_file());

        work_tree.make_dir(path)?;
        assert!(tmp.path().join(path).is_dir());
        assert!(!tmp.path().join(path).is_file());

        Ok(())
    }

    #[test]
    fn ignores_directory_from_fluxignore() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let repo_path = tmp.path();

        fs::write(repo_path.join(".fluxignore"), "target/\n")?;

        let work_tree = WorkTree::new(repo_path.to_path_buf());

        fs::write(repo_path.join("main.rs"), "fn main() {}")?;
        fs::create_dir(repo_path.join("target"))?;
        fs::write(repo_path.join("target").join("binary"), "binary")?;

        let mut files = work_tree.list_files(None)?;
        files.sort();

        assert_eq!(
            files,
            vec![PathBuf::from(".fluxignore"), PathBuf::from("main.rs")]
        );

        Ok(())
    }

    #[test]
    fn ignores_glob_pattern_from_gitignore() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let repo_path = tmp.path();

        fs::write(repo_path.join(".gitignore"), "*.log\n")?;

        let work_tree = WorkTree::new(repo_path.to_path_buf());

        fs::write(repo_path.join("main.rs"), "fn main() {}")?;
        fs::write(repo_path.join("debug.log"), "log data")?;
        fs::write(repo_path.join("error.log"), "log data")?;

        let mut files = work_tree.list_files(None)?;
        files.sort();

        assert_eq!(
            files,
            vec![PathBuf::from(".gitignore"), PathBuf::from("main.rs"),]
        );

        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn stat_permission_denied() -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempdir()?;
        let sub_dir = tmp.path().join("restricted_dir");
        fs::create_dir(&sub_dir)?;

        let secret = Path::new("restricted_dir/secret.txt");
        let full_path = tmp.path().join(secret);
        fs::write(&full_path, "data")?;

        fs::set_permissions(&sub_dir, fs::Permissions::from_mode(0o000))?;

        let work_tree = WorkTree::new(tmp.path().to_path_buf());
        let result = work_tree.stat_file(secret);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Permission denied")
        );

        fs::set_permissions(&sub_dir, fs::Permissions::from_mode(0o755))?;
        Ok(())
    }
}
