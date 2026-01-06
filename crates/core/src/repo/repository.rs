use crate::objects::blob::Blob;
use crate::objects::commit::Commit;
use crate::objects::object_type::{FluxObject, ObjectType};
use crate::objects::tree::Tree;
use crate::repo::branch::Branch;
use crate::repo::config::Config;
use crate::repo::index::Index;
use crate::repo::object_store::ObjectStore;
use crate::repo::work_tree::WorkTree;
use anyhow::{Context, bail};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

// TODO: 
// - finish refactoring
// - standardized error handling
// - cleanup cli commands and make them simpler
// - diff feature

pub struct Repository {
    pub work_tree: WorkTree,
    pub store_dir: PathBuf,
    pub config: Config,
    pub index: Index,
    pub head: String,
    pub branches: Vec<Branch>,
    pub object_store: ObjectStore,
}

impl Repository {
    fn load_branches(&mut self) -> anyhow::Result<()> {
        let heads_dir = self.store_dir.join("refs/heads");
        let current_branch_name = self.branch_name();

        let mut branches = Vec::new();

        for entry in fs::read_dir(&heads_dir)? {
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in branch name"))?;

            let ref_path = entry.path();

            let last_commit_hash = fs::read_to_string(&ref_path)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            branches.push(Branch {
                name: name.clone(),
                is_current: Some(&name) == current_branch_name.as_ref(),
                last_commit_hash,
                ref_path,
            });
        }

        self.branches = branches;
        Ok(())
    }

    pub fn branch_name(&self) -> Option<String> {
        self.head.strip_prefix("refs/heads/").map(String::from)
    }

    pub fn head_commit(&self) -> Option<String> {
        let branch_path = self.store_dir.join(&self.head);

        if !branch_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&branch_path).ok()?;

        let trimmed = content.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
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
        let blob = Blob::new(&path);
        self.object_store.store(&blob);

        let rel_path = path
            .strip_prefix(self.work_tree.path())
            .context("Path is outside work tree")?;

        let rel_path = rel_path.to_str().context("Non UTF-8 path")?;

        self.index.add(rel_path.into(), blob.hash())?;
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
        let work_tree_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let store_dir = work_tree_path.join(".flux");

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

        let mut repo = Self {
            work_tree: WorkTree::new(work_tree_path),
            object_store: ObjectStore::new(&store_dir),
            index,
            store_dir,
            config,
            head: "refs/heads/main".to_string(),
            branches: Vec::new(),
        };

        repo.load_branches()?;
        Ok(repo)
    }

    pub fn open(path: Option<String>) -> anyhow::Result<Self> {
        let work_tree_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let store_dir = work_tree_path.join(".flux");

        if !store_dir.exists() {
            bail!("Not a repository");
        }

        let config_path = store_dir.join("config");
        let config = Config::from(&config_path)?;
        let index = Index::load(&store_dir)?;

        let head_content = fs::read_to_string(store_dir.join("HEAD"))?;
        let head = if head_content.starts_with("ref: ") {
            head_content
                .trim()
                .strip_prefix("ref: ")
                .context("Invalid HEAD format")?
                .to_string()
        } else {
            bail!("Detached HEAD not supported");
        };

        let mut repo = Self {
            work_tree: WorkTree::new(work_tree_path),
            object_store: ObjectStore::load(&store_dir),
            store_dir,
            config,
            index,
            head,
            branches: Vec::new(),
        };

        repo.load_branches()?;
        Ok(repo)
    }

    pub fn set(&mut self, key: String, value: String) -> Result<(), anyhow::Error> {
        self.config.set(key, value)?;
        Ok(())
    }

    pub fn hash_object(&self, path: String, write: bool) -> anyhow::Result<String> {
        let full_path = self.work_tree.path().join(&path);
        let metadata = full_path.metadata().expect(&format!(
            "Could not read metadata for file: {:?}",
            full_path
        ));
        let object: Box<dyn FluxObject>;
        if metadata.is_file() {
            object = Box::new(Blob::new(&full_path));
        } else {
            object = Box::new(Tree::new(&full_path));
        }

        if write {
            self.object_store.store(object.as_ref());
        }

        Ok(object.hash())
    }

    pub fn cat(&self, object_hash: &str) -> anyhow::Result<()> {
        let object = self.object_store.retrieve_object(object_hash);
        object.print();

        Ok(())
    }

    pub fn commit_tree(
        &self,
        tree_hash: String,
        message: String,
        parent_hash: Option<String>,
    ) -> String {
        let (user_name, user_email) = self.config.get();
        let tree = self.object_store.retrieve_object(&tree_hash);

        if tree.object_type() != ObjectType::Tree {
            panic!("Cannot create a commit from something that is not a tree")
        }

        let commit = Commit::new(tree.hash(), user_name, user_email, parent_hash, message);
        self.object_store.store(&commit);
        commit.hash()
    }

    pub fn add(&mut self, path: &str) -> anyhow::Result<()> {
        let full_path = self.work_tree.path().join(path);
        self.add_path(&full_path)?;
        self.index.flush()?;
        Ok(())
    }

    pub fn delete(&mut self, path: &str) -> anyhow::Result<()> {
        let path = self.work_tree.path().join(path);
        if let Some(s) = path.to_str() {
            self.index.remove(s.into())?;
            self.index.flush()?;
        } else {
            bail!("Could not remove file from index.")
        }

        Ok(())
    }

    pub fn commit(&mut self, message: String) -> anyhow::Result<String> {
        if self.index.is_empty() {
            bail!("Nothing to commit");
        }

        let tree_hash = self
            .work_tree
            .build_tree_from_index(&self.index.map, &self.object_store)?;

        let (user_name, user_email) = self.config.get();
        let parent = self.head_commit();

        let commit = Commit::new(tree_hash, user_name, user_email, parent, message);

        self.object_store.store(&commit);

        let branch_path = self.store_dir.join(&self.head);
        fs::write(branch_path, &commit.hash())?;
        self.index.clear()?;

        Ok(commit.hash())
    }

    pub fn log(&self, _reference: Option<String>) -> anyhow::Result<()> {
        let mut current_hash = self.head_commit();
        while let Some(hash) = current_hash {
            self.cat(&hash)?;
            let current = self.object_store.retrieve_object(&hash);
            if let Some(commit) = current.as_any().downcast_ref::<Commit>() {
                current_hash = commit.parent_hash().map(String::from);
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn switch_branch(&mut self, branch_name: &str, force: bool) -> anyhow::Result<()> {
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
        self.head = branch_ref;

        self.work_tree.clear()?;
        if let Some(commit_hash) = self.head_commit() {
            self.work_tree
                .restore_from_commit(&commit_hash, &self.object_store)?;
        }

        self.load_branches()?;
        Ok(())
    }

    pub fn new_branch(&mut self, branch_name: &str) -> anyhow::Result<()> {
        let branch_ref = format!("refs/heads/{}", branch_name);
        let branch_head_path = self.store_dir.join(&branch_ref);

        if branch_head_path.exists() {
            bail!("Branch '{}' already exists", branch_name);
        }

        File::create(&branch_head_path)?;

        if let Some(commit_hash) = self.head_commit() {
            fs::write(&branch_head_path, commit_hash)?;
        }

        fs::write(
            self.store_dir.join("HEAD"),
            format!("ref: {}\n", &branch_ref),
        )?;
        self.head = branch_ref;

        self.load_branches()?;
        Ok(())
    }

    pub fn list_branches(&self) -> Vec<String> {
        self.branches
            .iter()
            .map(|b| {
                if b.is_current {
                    format!("(*) {}", b.name)
                } else {
                    format!("    {}", b.name)
                }
            })
            .collect()
    }
}
