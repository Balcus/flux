use crate::error;
use crate::internals::config::Config;
use crate::internals::index::Index;
use crate::internals::object_store::ObjectStore;
use crate::internals::refs::Refs;
use crate::internals::work_tree::WorkTree;
use crate::objects::blob::Blob;
use crate::objects::commit::Commit;
use crate::objects::object_type::{FluxObject, ObjectType};
use crate::objects::tree::Tree;
use anyhow::{Context, bail};
use std::fs;
use std::path::{Path, PathBuf};

// TODO:
// - standardized error handling + tests
// - cleanup cli commands and make them simpler
// - diff feature

// KEEP IN MINDS:
// - sanitize file names given by user

pub struct Repository {
    pub refs: Refs,
    pub work_tree: WorkTree,
    pub flux_dir: PathBuf,
    pub config: Config,
    pub index: Index,
    pub object_store: ObjectStore,
}

impl Repository {
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
        self.object_store.store(&blob).map_err(|e| {
            error::RepositoryError::with_context(
                "failed adding file to index, could not store object",
                e,
            )
        })?;

        let rel_path = path.strip_prefix(self.work_tree.path()).with_context(|| {
            format!(
                "add: '{}' is outside work tree '{}'",
                path.display(),
                self.work_tree.path().display(),
            )
        })?;

        let rel_str = rel_path
            .to_str()
            .with_context(|| format!("add command failed for non UTF8 path: '{:?}'", rel_path))?;

        self.index
            .add(rel_str.to_owned(), blob.hash())
            .with_context(|| format!("add command failed to update index for: '{}'", rel_str))?;

        Ok(())
    }

    pub fn init(path: Option<String>, force: bool) -> Result<Self, error::RepositoryError> {
        let work_tree_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let flux_dir = work_tree_path.join(".flux");

        if flux_dir.exists() && force {
            fs::remove_dir_all(&flux_dir).map_err(|e| {
                error::RepositoryError::with_context("failed to force reinitialize repository", e)
            })?;
        } else if flux_dir.exists() && !force {
            return Err(error::RepositoryError::AlreadyInitialized(flux_dir.clone()));
        }

        fs::create_dir_all(&flux_dir).map_err(|e| {
            error::RepositoryError::with_context("failed to initalize repository", e)
        })?;

        let object_store = ObjectStore::new(&flux_dir).map_err(|e| {
            error::RepositoryError::with_context(
                "failed to initalize object store for repository",
                e,
            )
        })?;

        let refs = Refs::new(&flux_dir);

        let config = Config::default(&flux_dir.join("config")).map_err(|e| {
            error::RepositoryError::with_context("faliled to create config for repository", e)
        })?;

        let index = Index::new(&flux_dir).map_err(|e| {
            error::RepositoryError::with_context("failed to initalize index for repository", e)
        })?;
        let work_tree = WorkTree::new(work_tree_path);

        let repo = Self {
            work_tree,
            object_store,
            index,
            flux_dir,
            config,
            refs,
        };

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
        let object_store = ObjectStore::load(&store_dir)
            .with_context(|| format!("failed to load object store"))?;

        Ok(Self {
            refs: Refs::load(&store_dir),
            work_tree: WorkTree::new(work_tree_path),
            object_store,
            flux_dir: store_dir,
            config,
            index,
        })
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
            self.object_store.store(object.as_ref()).map_err(|e| {
                error::RepositoryError::with_context(
                    "hash-object failed, could not store object",
                    e,
                )
            })?;
        }

        Ok(object.hash())
    }

    pub fn cat(&self, object_hash: &str) -> Result<(), error::RepositoryError> {
        let object = self
            .object_store
            .retrieve_object(object_hash)
            .map_err(|e| {
                error::RepositoryError::with_context(
                    "cat command failed, could not retrieve object from object store",
                    e,
                )
            })?;
        object.print();

        Ok(())
    }

    pub fn commit_tree(
        &self,
        tree_hash: String,
        message: String,
        parent_hash: Option<String>,
    ) -> Result<String, error::RepositoryError> {
        let (user_name, user_email) = self.config.get().map_err(|e| {
            error::RepositoryError::with_context(
                "commit-tree failed, could not read user related fields from configuration",
                e,
            )
        })?;

        let tree = self.object_store.retrieve_object(&tree_hash).map_err(|e| {
            error::RepositoryError::with_context(
                "commit-tree command failed, could not load tree from object store",
                e,
            )
        })?;

        if tree.object_type() != ObjectType::Tree {
            return Err(error::RepositoryError::CommitRoot { hash: tree.hash() });
        }

        let commit = Commit::new(tree.hash(), user_name, user_email, parent_hash, message);
        self.object_store.store(&commit).map_err(|e| {
            error::RepositoryError::with_context("failed commit-tree, could not store object", e)
        })?;
        Ok(commit.hash())
    }

    pub fn add(&mut self, path: &str) -> anyhow::Result<()> {
        let full_path = self.work_tree.path().join(path);
        self.add_path(&full_path)?;
        self.index.flush()?;
        Ok(())
    }

    pub fn delete(&mut self, rel: &str) -> anyhow::Result<()> {
        let abs = self.work_tree.path().join(rel);

        let key = abs.to_str().with_context(|| {
            format!("delete command failed for non UTF8 path: {}", abs.display())
        })?;

        let removed = self
            .index
            .remove(key)
            .with_context(|| format!("failed to update index on key: {key} deletion"))?;

        if !removed {
            eprint!("warning: {key} is not tracked");
        }

        Ok(())
    }

    pub fn commit(&mut self, message: String) -> Result<String, error::RepositoryError> {
        if self.index.is_empty() {
            return Err(error::RepositoryError::IndexEmpty);
        }

        let tree_hash = self
            .work_tree
            .build_tree_from_index(&self.index.map, &self.object_store)
            .map_err(|e| {
                error::RepositoryError::with_context(
                    "failed to create commit, could not build tree from index",
                    e,
                )
            })?;

        let (user_name, user_email) = self.config.get().map_err(|e| {
            error::RepositoryError::with_context(
                "failed to create commit, could not read user related fields from configuration",
                e,
            )
        })?;

        // change
        let last = self.refs.head_commit().unwrap();
        let parent = (!last.is_empty()).then_some(last);

        let commit = Commit::new(tree_hash, user_name, user_email, parent, message);

        self.object_store.store(&commit).map_err(|e| {
            error::RepositoryError::with_context("commit failed, could not store object", e)
        })?;

        let hash = commit.hash();

        //change
        self.refs.update_head(&hash).unwrap();

        //change
        self.index.clear().unwrap();

        Ok(hash)
    }

    pub fn log(&self, _reference: Option<String>) -> Result<(), error::RepositoryError> {
        let mut current_hash = self.refs.head_commit().ok().filter(|s| !s.is_empty());
        while let Some(hash) = current_hash {
            self.cat(&hash)?;
            let current = self.object_store.retrieve_object(&hash).map_err(|e| {
                error::RepositoryError::with_context(
                    "log command failed, could not retrieve objects from object store",
                    e,
                )
            })?;
            if let Some(commit) = current.as_any().downcast_ref::<Commit>() {
                current_hash = commit.parent_hash().map(String::from);
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn show_branches(&self) -> anyhow::Result<String> {
        self.refs.format_branches()
    }

    pub fn list_branches(&self) -> anyhow::Result<Vec<String>> {
        self.refs.list_branches()
    }

    pub fn new_branch(&mut self, name: &str) {
        self.refs.new_branch(name);
    }

    pub fn switch_branch(&mut self, name: &str, force: bool) {
        if self.has_uncommitted_changes() && !force {
            println!("Failed to switch branch, repository has uncommited changes");
            return;
        }

        if let Err(e) = self.refs.switch_branch(name) {
            println!("Failed to switch branches: {e}");
            return;
        }

        if let Err(e) = self.index.clear() {
            println!("Failed to clear index: {e}");
            return;
        }

        if let Err(e) = self.work_tree.clear() {
            println!("Failed to clear work tree: {e}");
            return;
        }

        match self.refs.head_commit() {
            Ok(commit_hash) if !commit_hash.is_empty() => {
                if let Err(e) = self
                    .work_tree
                    .restore_from_commit(&commit_hash, &self.object_store)
                {
                    println!("Failed to restore work tree from commit {commit_hash}: {e}");
                    return;
                }
            }
            Ok(_) => {}
            Err(e) => {
                println!("Failed to resolve branch head commit: {e}");
                return;
            }
        }
    }
}
