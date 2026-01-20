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
// - finish refactoring
// - standardized error handling
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
        self.object_store.store(&blob);

        let rel_path = path
            .strip_prefix(self.work_tree.path())
            .context("Path is outside work tree")?;

        let rel_path = rel_path.to_str().context("Non UTF-8 path")?;

        self.index.add(rel_path.into(), blob.hash())?;
        Ok(())
    }

    pub fn init(path: Option<String>, force: bool) -> anyhow::Result<Self> {
        let work_tree_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let flux_dir = work_tree_path.join(".flux");

        if flux_dir.join("config").exists() && !force {
            bail!("Repository already initialized");
        }

        fs::create_dir_all(&flux_dir)?;
        let object_store = ObjectStore::new(&flux_dir);
        let refs = Refs::new(&flux_dir);
        let config = Config::default(&flux_dir.join("config"))?;
        let index = Index::new(&flux_dir)?;
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

        Ok(Self {
            refs: Refs::load(&store_dir),
            work_tree: WorkTree::new(work_tree_path),
            object_store: ObjectStore::load(&store_dir),
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
        use anyhow::bail;

        if self.index.is_empty() {
            bail!("Nothing to commit");
        }

        let tree_hash = self
            .work_tree
            .build_tree_from_index(&self.index.map, &self.object_store)?;

        let (user_name, user_email) = self.config.get();

        let last = self.refs.head_commit()?;
        let parent = (!last.is_empty()).then_some(last);

        let commit = Commit::new(tree_hash, user_name, user_email, parent, message);

        self.object_store.store(&commit);

        let hash = commit.hash();
        self.refs.update_head(&hash)?;

        self.index.clear()?;

        Ok(hash)
    }

    pub fn log(&self, _reference: Option<String>) -> anyhow::Result<()> {
        let mut current_hash = self.refs.head_commit().ok().filter(|s| !s.is_empty());
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
