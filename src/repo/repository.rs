use crate::objects::{commit, tree};
use crate::repo::config::Config;
use crate::repo::index::Index;
use crate::shared::types::object_type::ObjectType;
use crate::shared::types::tree_entry::TreeEntry;
use crate::utils;
use crate::utils::write_object;
use anyhow::{Context, bail};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

pub struct Repository {
    pub work_tree: PathBuf,
    pub store_dir: PathBuf,
    pub config: Config,
    pub index: Index,
    pub branch: String,
}

impl Repository {
    fn head_commit(&self) -> anyhow::Result<Option<String>> {
        let branch_path = self.store_dir.join(&self.branch);
        Ok(fs::read_to_string(branch_path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()))
    }

    fn branch_name(&self) -> Option<String> {
        self.branch.strip_prefix("refs/heads/").map(String::from)
    }

    fn restore_working_tree(&self, commit_hash: &str) -> anyhow::Result<()> {
        let commit = utils::read_object(&self.store_dir, commit_hash)?;

        if commit.object_type != ObjectType::Commit {
            bail!("Expected commit object");
        }

        let content = String::from_utf8(commit.decompressed_content)?;
        let tree_hash = content
            .lines()
            .find(|line| line.starts_with("tree "))
            .and_then(|line| line.strip_prefix("tree "))
            .context("Commit has no tree")?
            .trim()
            .to_string();

        self.restore_tree(&tree_hash, &self.work_tree)?;

        Ok(())
    }

    fn restore_tree(&self, tree_hash: &str, target_dir: &Path) -> anyhow::Result<()> {
        let entries = tree::parse_tree(&self.store_dir, tree_hash)?;

        for entry in entries {
            let target_path = target_dir.join(&entry.name);

            if entry.mode.starts_with("040") {
                fs::create_dir_all(&target_path)?;
                self.restore_tree(&entry.hash, &target_path)?;
            } else {
                let blob = utils::read_object(&self.store_dir, &entry.hash)?;

                if blob.object_type != ObjectType::Blob {
                    bail!("Expected blob object");
                }

                fs::write(&target_path, blob.decompressed_content)?;
            }
        }

        Ok(())
    }

    fn clear_working_tree(&self) -> anyhow::Result<()> {
        for entry in fs::read_dir(&self.work_tree)? {
            let entry = entry?;
            let path = entry.path();

            if path.file_name().and_then(|n| n.to_str()) == Some(".flux") {
                continue;
            }

            if path.is_file() {
                fs::remove_file(path)?;
            } else if path.is_dir() {
                fs::remove_dir_all(path)?;
            }
        }

        Ok(())
    }

    fn has_uncommitted_changes(&self) -> bool {
        !self.index.is_empty()
    }

    fn add_path(&mut self, path: &Path) -> anyhow::Result<()> {
        let metadata = fs::metadata(path)?;

        if metadata.is_file() {
            self.add_file(path)?;
        } else if metadata.is_dir() {
            if path.ends_with(".flux") {
                return Ok(());
            }

            for entry in fs::read_dir(path)? {
                let entry = entry?;
                self.add_path(&entry.path())?;
            }
        }

        Ok(())
    }

    fn add_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let res = utils::write_object(&self.store_dir, &self.work_tree, path)?;

        let rel_path = path
            .strip_prefix(&self.work_tree)
            .context("Path is outside work tree")?;

        let rel_path = rel_path.to_str().context("Non UTF-8 path")?;

        self.index.add(rel_path.into(), res.hash)?;
        Ok(())
    }

    pub fn show_branches(&self) -> anyhow::Result<String> {
        let files = fs::read_dir(self.store_dir.join("refs/heads"))
            .context("Could not open refs/heads directory")?;

        let current = self.branch_name();

        let mut res = String::new();

        for file in files {
            let file = file.context("Could not read branch entry")?;

            let name = file
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in branch name"))?;

            if Some(&name) == current.as_ref() {
                res.push_str("(*) ");
            } else {
                res.push_str("  ");
            }

            res.push_str(&name);
            res.push('\n');
        }

        Ok(res)
    }

    pub fn init(path: Option<String>, force: bool) -> anyhow::Result<Self> {
        let work_tree = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let store_dir = work_tree.join(".flux");

        if store_dir.join("config").exists() && !force {
            bail!("Repository already initialized");
        }

        fs::create_dir_all(&store_dir)?;
        fs::create_dir(&store_dir.join("objects"))?;
        fs::create_dir(&store_dir.join("refs"))?;
        fs::create_dir(&store_dir.join("refs/heads"))?;
        File::create(&store_dir.join("refs/heads/main"))?;
        let config = Config::default(&store_dir.join("config"))?;
        fs::write(&store_dir.join("HEAD"), "ref: refs/heads/main\n")?;
        fs::write(&store_dir.join("index"), "{}")?;
        let index = Index::empty(&store_dir)?;
        println!("Initialized repository");

        Ok(Self {
            work_tree,
            index,
            store_dir,
            config,
            branch: "refs/heads/main".to_string(),
        })
    }

    pub fn open(path: Option<String>) -> anyhow::Result<Self> {
        let work_tree = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let store_dir = work_tree.join(".flux");

        if !store_dir.exists() {
            bail!("Not a git repository");
        }

        let config_path = store_dir.join("config");
        let config = Config::from(&config_path)?;
        let index = Index::load(&store_dir)?;

        let head_content = fs::read_to_string(store_dir.join("HEAD"))?;
        let branch = if head_content.starts_with("ref: ") {
            head_content
                .trim()
                .strip_prefix("ref: ")
                .context("Invalid HEAD format")?
                .to_string()
        } else {
            bail!("Detached HEAD not supported");
        };

        Ok(Self {
            config,
            work_tree,
            store_dir,
            index,
            branch,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<(), anyhow::Error> {
        self.config.set(key, value)?;
        Ok(())
    }

    pub fn hash_object(&self, path: String, write: bool) -> anyhow::Result<String> {
        let full_path = self.work_tree.join(&path);
        let hash = if write {
            let result = write_object(&self.store_dir, &self.work_tree, &full_path)?;
            result.hash
        } else {
            utils::get_hash(&self.store_dir, &self.work_tree, &full_path)?
        };

        Ok(hash)
    }

    pub fn cat_file(&self, object_hash: &str) -> anyhow::Result<()> {
        let object = utils::read_object(&self.store_dir, object_hash)?;

        match object.object_type {
            ObjectType::Blob => {
                let output = String::from_utf8(object.decompressed_content)
                    .context("Blob contains invalid UTF-8")?;
                print!("{}", output);
            }
            ObjectType::Tree => {
                self.ls_tree(object_hash)?;
            }
            ObjectType::Commit => {
                commit::show_commit(&self.store_dir, object_hash)?;
            }
            _ => bail!("cat_file currently supports only blob objects"),
        }

        Ok(())
    }

    pub fn ls_tree(&self, tree_hash: &str) -> anyhow::Result<String> {
        Ok(tree::ls_tree(&self.store_dir, tree_hash)?)
    }

    pub fn commit_tree(
        &self,
        tree_hash: String,
        message: String,
        parent_hash: Option<String>,
    ) -> anyhow::Result<String> {
        let (user_name, user_email) = self.config.get();
        let object = utils::read_object(&self.store_dir, &tree_hash)?;
        let hash = match object.object_type {
            ObjectType::Tree => commit::commit_tree(
                &self.store_dir,
                user_name,
                user_email,
                tree_hash,
                parent_hash,
                message,
            )?,
            _ => bail!("Can only commit tree objects"),
        };
        Ok(hash)
    }

    pub fn add(&mut self, path: &str) -> anyhow::Result<()> {
        let full_path = self.work_tree.join(path);
        self.add_path(&full_path)?;
        self.index.flush()?;
        Ok(())
    }

    pub fn delete(&mut self, path: &str) -> anyhow::Result<()> {
        let path = self.work_tree.join(path);
        if let Some(s) = path.to_str() {
            self.index.remove(s.into())?;
            self.index.flush()?;
        } else {
            bail!("Could not remove file from index.")
        }

        Ok(())
    }

    pub fn tree_from_index(&self) -> anyhow::Result<String> {
        let mut entries = Vec::new();

        for (path, hash) in &self.index.map {
            let name = Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .context("Invalid filename in index")?
                .to_string();

            let object = utils::read_object(&self.store_dir, hash)?;

            let (mode, entry_type) = match object.object_type {
                ObjectType::Blob => ("100644".to_string(), "blob".to_string()),
                ObjectType::Tree => ("040000".to_string(), "tree".to_string()),
                _ => bail!("Unexpected object type in index"),
            };

            entries.push(TreeEntry {
                mode,
                entry_type,
                hash: hash.clone(),
                name,
            });
        }

        let tree_content = tree::build_tree_content(entries);
        let result = tree::hash_tree(tree_content)?;
        utils::store_object(
            &self.store_dir,
            &result.object_hash,
            &result.compressed_content,
        )?;

        Ok(result.object_hash)
    }

    pub fn commit(&mut self, message: String) -> anyhow::Result<String> {
        if self.index.is_empty() {
            bail!("Nothing to commit");
        }

        let index_tree_hash = self.tree_from_index()?;
        let (user_name, user_email) = self.config.get();
        let parent = self.head_commit()?;

        let commit_hash = commit::commit_tree(
            &self.store_dir,
            user_name,
            user_email,
            index_tree_hash,
            parent,
            message,
        )?;

        let branch_path = self.store_dir.join(&self.branch);
        fs::write(branch_path, &commit_hash)?;
        self.index.clear()?;

        Ok(commit_hash)
    }

    pub fn log(&self, _reference: Option<String>) -> anyhow::Result<()> {
        let mut current = self.head_commit()?;

        while let Some(hash) = current {
            self.cat_file(&hash)?;
            current = commit::get_parent_hash(&self.store_dir, hash)?;
        }

        Ok(())
    }

    pub fn switch_branch(&mut self, branch_name: String, force: bool) -> anyhow::Result<()> {
        let branch_ref = format!("refs/heads/{}", branch_name);
        let branch_path = self.store_dir.join(&branch_ref);

        if !branch_path.exists() {
            bail!("Branch '{}' does not exist", branch_name);
        }

        if self.has_uncommitted_changes() && !force {
            bail!("The current branch has uncommited changes");
        }

        fs::write(
            self.store_dir.join("HEAD"),
            format!("ref: {}\n", branch_ref),
        )?;

        self.branch = branch_ref.clone();

        if let Some(commit_hash) = self.head_commit()? {
            self.restore_working_tree(&commit_hash)?;
        } else {
            self.clear_working_tree()?;
        }

        Ok(())
    }

    pub fn new_branch(&self, brach_name: String) -> anyhow::Result<()> {
        let branch_ref = format!("refs/heads/{}", brach_name);
        let branch_head_path = &self.store_dir.join(&branch_ref);
        File::create(branch_head_path)?;
        if let Some(commit_hash) = &self.head_commit()? {
            fs::write(branch_head_path, commit_hash)?;
        }
        fs::write(
            self.store_dir.join("HEAD"),
            format!("ref: {}\n", &branch_ref),
        )?;
        Ok(())
    }
}
