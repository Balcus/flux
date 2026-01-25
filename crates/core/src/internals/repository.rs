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
use std::fs;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, error::RepositoryError>;

#[derive(Debug)]
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

    fn add_path(&mut self, path: &Path) -> Result<()> {
        let metadata = fs::metadata(path).map_err(|e| error::IoError::Metadata {
            path: path.to_owned(),
            source: e,
        })?;

        if metadata.is_file() {
            self.add_file(path)?;
        } else if metadata.is_dir() {
            if path.ends_with(".flux") {
                return Ok(());
            }

            let iter = fs::read_dir(path).map_err(|e| error::IoError::Read {
                path: path.to_owned(),
                source: e,
            })?;

            for entry in iter {
                let entry = entry.map_err(|e| error::IoError::Read {
                    path: path.to_owned(),
                    source: e,
                })?;
                self.add_path(&entry.path())?;
            }
        }

        Ok(())
    }

    fn add_file(&mut self, path: &Path) -> Result<()> {
        let blob = Blob::new(&path);
        self.object_store.store(&blob).map_err(|e| {
            error::RepositoryError::with_context(
                "Failed adding file to index, could not store object",
                e,
            )
        })?;

        let rel_path = path.strip_prefix(self.work_tree.path()).map_err(|e| {
            error::RepositoryError::with_context(
                "Failed to strip prefix from file. file might be outisde of the working directory",
                e,
            )
        })?;

        let rel_str = rel_path
            .to_str()
            .ok_or_else(|| error::RepositoryError::PathName {
                path: rel_path.to_owned(),
            })?;

        self.index
            .add(rel_str.to_owned(), blob.hash())
            .map_err(|e| error::RepositoryError::with_context("Failed to add file to index", e))?;

        Ok(())
    }

    pub fn init(path: Option<String>, force: bool) -> Result<Self> {
        let work_tree_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let flux_dir = work_tree_path.join(".flux");

        if flux_dir.exists() && force {
            fs::remove_dir_all(&flux_dir).map_err(|e| {
                error::RepositoryError::with_context("Failed to force reinitialize repository", e)
            })?;
        } else if flux_dir.exists() && !force {
            let abs = flux_dir.canonicalize().unwrap_or_else(|_| flux_dir.clone());
            return Err(error::RepositoryError::AlreadyInitialized(abs));
        }

        fs::create_dir_all(&flux_dir).map_err(|e| {
            error::RepositoryError::with_context("Failed to initalize repository", e)
        })?;

        let object_store = ObjectStore::new(&flux_dir).map_err(|e| {
            error::RepositoryError::with_context(
                "Failed to initalize object store for repository",
                e,
            )
        })?;

        let refs = Refs::new(&flux_dir).map_err(|e| {
            error::RepositoryError::with_context("Failed to initalize refs for repository", e)
        })?;

        let config = Config::default(&flux_dir.join("config")).map_err(|e| {
            error::RepositoryError::with_context("Faliled to create config for repository", e)
        })?;

        let index = Index::new(&flux_dir).map_err(|e| {
            error::RepositoryError::with_context("Failed to initalize index for repository", e)
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

    pub fn open(path: Option<String>) -> Result<Self> {
        let work_tree_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let store_dir = work_tree_path.join(".flux");

        if !store_dir.exists() {
            let abs = work_tree_path
                .canonicalize()
                .unwrap_or_else(|_| work_tree_path.clone());
            return Err(error::RepositoryError::NotRepository(abs));
        }

        let config_path = store_dir.join("config");
        let config = Config::from(&config_path).map_err(|e| {
            error::RepositoryError::with_context(
                "Failed to open repository, could not load configuration.",
                e,
            )
        })?;
        let index = Index::load(&store_dir).map_err(|e| {
            error::RepositoryError::with_context(
                "Failed to open repository, could not load index.",
                e,
            )
        })?;
        let object_store = ObjectStore::load(&store_dir).map_err(|e| {
            error::RepositoryError::with_context(
                "Failed to open repository, could not load object store.",
                e,
            )
        })?;
        let refs = Refs::load(&store_dir).map_err(|e| {
            error::RepositoryError::with_context(
                "Failed to open repository, could not load refs.",
                e,
            )
        })?;

        Ok(Self {
            refs,
            work_tree: WorkTree::new(work_tree_path),
            object_store,
            flux_dir: store_dir,
            config,
            index,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.config.set(key, value).map_err(|e| {
            error::RepositoryError::with_context("Failed to set key in config file.", e)
        })?;
        Ok(())
    }

    pub fn hash_object(&self, path: String, write: bool) -> Result<String> {
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
                    "Hash-object failed, could not store object",
                    e,
                )
            })?;
        }

        Ok(object.hash())
    }

    pub fn cat(&self, object_hash: &str) -> Result<()> {
        let object = self
            .object_store
            .retrieve_object(object_hash)
            .map_err(|e| {
                error::RepositoryError::with_context(
                    "Cat command failed, could not retrieve object from object store",
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
    ) -> Result<String> {
        let (user_name, user_email) = self.config.get().map_err(|e| {
            error::RepositoryError::with_context(
                "Commit-tree failed, could not read user related fields from configuration",
                e,
            )
        })?;

        let tree = self.object_store.retrieve_object(&tree_hash).map_err(|e| {
            error::RepositoryError::with_context(
                "Commit-tree command failed, could not load tree from object store",
                e,
            )
        })?;

        if tree.object_type() != ObjectType::Tree {
            return Err(error::RepositoryError::CommitRoot { hash: tree.hash() });
        }

        let commit = Commit::new(tree.hash(), user_name, user_email, parent_hash, message);
        self.object_store.store(&commit).map_err(|e| {
            error::RepositoryError::with_context("Failed commit-tree, could not store object", e)
        })?;
        Ok(commit.hash())
    }

    pub fn add(&mut self, path: &str) -> Result<()> {
        let full_path = self.work_tree.path().join(path);
        self.add_path(&full_path)?;
        Ok(())
    }

    pub fn delete(&mut self, rel: &str) -> Result<()> {
        let abs = self.work_tree.path().join(rel);

        let key = abs
            .to_str()
            .ok_or_else(|| error::RepositoryError::PathName { path: abs.clone() })?;

        let removed = self.index.remove(key).map_err(|e| {
            error::RepositoryError::with_context(
                "Failed command delete, could not remove file from index.",
                e,
            )
        })?;

        if !removed {
            eprint!("warning: {key} is not tracked");
        }

        Ok(())
    }

    pub fn commit(&mut self, message: String) -> Result<String> {
        if self.index.is_empty() {
            return Err(error::RepositoryError::IndexEmpty);
        }

        let tree_hash = self
            .work_tree
            .build_tree_from_index(&self.index.map, &self.object_store)
            .map_err(|e| {
                error::RepositoryError::with_context(
                    "Failed to create commit, could not build tree from index",
                    e,
                )
            })?;

        let (user_name, user_email) = self.config.get().map_err(|e| {
            error::RepositoryError::with_context(
                "Failed to create commit, could not read user related fields from configuration",
                e,
            )
        })?;

        let last = self.refs.head_commit().unwrap();
        let parent = (!last.is_empty()).then_some(last);

        let commit = Commit::new(tree_hash, user_name, user_email, parent, message);

        self.object_store.store(&commit).map_err(|e| {
            error::RepositoryError::with_context("Commit failed, could not store object", e)
        })?;

        let hash = commit.hash();

        self.refs.update_head(&hash).unwrap();
        self.index.clear().unwrap();

        Ok(hash)
    }

    pub fn log(&self, _reference: Option<String>) -> Result<()> {
        let mut current_hash = self.refs.head_commit().ok().filter(|s| !s.is_empty());
        while let Some(hash) = current_hash {
            self.cat(&hash)?;
            let current = self.object_store.retrieve_object(&hash).map_err(|e| {
                error::RepositoryError::with_context(
                    "Log command failed, could not retrieve objects from object store",
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

    pub fn show_branches(&self) -> Result<String> {
        let branches = self
            .refs
            .format_branches()
            .map_err(|e| error::RepositoryError::with_context("Failed to show brances", e))?;
        Ok(branches)
    }

    pub fn list_branches(&self) -> Result<Vec<String>> {
        let branches = self
            .refs
            .list_branches()
            .map_err(|e| error::RepositoryError::with_context("Failed to show branches.", e))?;
        Ok(branches)
    }

    pub fn new_branch(&mut self, name: &str) -> Result<()> {
        self.refs
            .new_branch(name)
            .map_err(|e| error::RepositoryError::with_context("Failed to create new branch.", e))?;
        Ok(())
    }

    pub fn switch_branch(&mut self, name: &str, force: bool) -> Result<()> {
        if self.has_uncommitted_changes() && !force {
            return Err(error::RepositoryError::UncommitedChanges);
        }

        self.refs
            .switch_branch(name)
            .map_err(|e| error::RepositoryError::with_context("Switching branches failed.", e))?;

        self.index.clear().map_err(|e| {
            error::RepositoryError::with_context(
                "Switching branches failed, could not clear index.",
                e,
            )
        })?;

        self.work_tree.clear().map_err(|e| {
            error::RepositoryError::with_context(
                "Switching branches failed, could not clear working tree.",
                e,
            )
        })?;

        let commit = self.refs.head_commit().map_err(|e| {
            error::RepositoryError::with_context(
                "Switching branches failed, could not resolve head for the new branch.",
                e,
            )
        })?;

        if !commit.is_empty() {
            self.work_tree
                .restore_from_commit(&commit, &self.object_store)
                .map_err(|e| {
                    error::RepositoryError::with_context(
                        "Switching branches failed, could not restore worktree to match new branch",
                        e,
                    )
                })?;
        }

        Ok(())
    }
}
