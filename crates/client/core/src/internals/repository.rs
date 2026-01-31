use crate::error;
use crate::internals::config::Config;
use crate::internals::grpc_client::GrpcClient;
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
        let metadata = fs::metadata(path).map_err(|e| error::IoError::metadata_error(path, e))?;

        if metadata.is_file() {
            self.add_file(path)?;
        } else if metadata.is_dir() {
            if path.ends_with(".flux") {
                return Ok(());
            }

            let iter = fs::read_dir(path).map_err(|e| error::IoError::read_error(path, e))?;
            for entry in iter {
                let entry = entry.map_err(|e| error::IoError::read_error(path, e))?;
                self.add_path(&entry.path())?;
            }
        }

        Ok(())
    }

    fn add_file(&mut self, path: &Path) -> Result<()> {
        let blob = Blob::new(&path);
        self.object_store.store(&blob)?;

        let rel_path = path.strip_prefix(self.work_tree.path()).map_err(|e| {
            error::RepositoryError::from(
                "Failed to strip prefix from file. file might be outisde of the working directory",
                e,
            )
        })?;

        let rel_str = rel_path
            .to_str()
            .ok_or_else(|| error::RepositoryError::PathName {
                path: rel_path.to_owned(),
            })?;

        self.index.add(rel_str.to_owned(), blob.hash())?;

        Ok(())
    }

    pub fn init(path: Option<String>, force: bool) -> Result<Self> {
        let work_tree_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let flux_dir = work_tree_path.join(".flux");

        if flux_dir.exists() && force {
            fs::remove_dir_all(&flux_dir)
                .map_err(|e| error::IoError::delete_error(&flux_dir, e))?;
        } else if flux_dir.exists() && !force {
            let abs = flux_dir.canonicalize().unwrap_or_else(|_| flux_dir.clone());
            return Err(error::RepositoryError::AlreadyInitialized(abs));
        }

        fs::create_dir_all(&flux_dir).map_err(|e| error::IoError::create_error(&flux_dir, e))?;

        let object_store = ObjectStore::new(&flux_dir)?;
        let refs = Refs::new(&flux_dir)?;
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
        let config = Config::from(&config_path)?;
        let index = Index::load(&store_dir)?;
        let object_store = ObjectStore::load(&store_dir)?;
        let refs = Refs::load(&store_dir)?;

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
        self.config.set(key, value)?;
        Ok(())
    }

    pub fn hash_object(&self, path: String, write: bool) -> Result<String> {
        let full_path = self.work_tree.path().join(&path);
        let metadata = full_path
            .metadata()
            .map_err(|e| error::IoError::metadata_error(&full_path, e))?;
        let object: Box<dyn FluxObject>;
        if metadata.is_file() {
            object = Box::new(Blob::new(&full_path));
        } else {
            object = Box::new(Tree::new(&full_path));
        }

        if write {
            self.object_store.store(object.as_ref())?;
        }

        Ok(object.hash())
    }

    pub fn cat(&self, object_hash: &str) -> Result<()> {
        let object = self.object_store.retrieve_object(object_hash)?;
        object.print();

        Ok(())
    }

    pub fn commit_tree(
        &self,
        tree_hash: String,
        message: String,
        parent_hash: Option<String>,
    ) -> Result<String> {
        let credentials = self.config.get_credential();

        let user_name = credentials
            .user_name
            .ok_or_else(|| error::RepositoryError::Credentials())?;
        let user_email = credentials
            .user_email
            .ok_or_else(|| error::RepositoryError::Credentials())?;

        let tree = self.object_store.retrieve_object(&tree_hash)?;

        if tree.object_type() != ObjectType::Tree {
            return Err(error::RepositoryError::CommitRoot { hash: tree.hash() });
        }

        let commit = Commit::new(tree.hash(), user_name, user_email, parent_hash, message);
        self.object_store.store(&commit)?;
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

        let removed = self.index.remove(key)?;

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
            .build_tree_from_index(&self.index.map, &self.object_store)?;

        let credentials = self.config.get_credential();
        let user_name = credentials
            .user_name
            .ok_or_else(|| error::RepositoryError::Credentials())?;
        let user_email = credentials
            .user_email
            .ok_or_else(|| error::RepositoryError::Credentials())?;

        let last = self.refs.head_commit()?;
        let parent = (!last.is_empty()).then_some(last);
        let commit = Commit::new(tree_hash, user_name, user_email, parent, message);
        self.object_store.store(&commit)?;
        let hash = commit.hash();
        self.refs.update_head(&hash)?;
        self.index.clear()?;

        Ok(hash)
    }

    pub fn log(&self, _reference: Option<String>) -> Result<()> {
        let mut current_hash = self.refs.head_commit().ok().filter(|s| !s.is_empty());

        while let Some(hash) = current_hash {
            self.cat(&hash)?;
            let current = self.object_store.retrieve_object(&hash)?;
            if let Some(commit) = current.as_any().downcast_ref::<Commit>() {
                current_hash = commit.parent_hash().map(String::from);
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn show_branches(&self) -> Result<String> {
        let branches = self.refs.format_branches()?;
        Ok(branches)
    }

    pub fn list_branches(&self) -> Result<Vec<String>> {
        let branches = self.refs.list_branches()?;
        Ok(branches)
    }

    pub fn new_branch(&mut self, name: &str) -> Result<()> {
        self.refs.new_branch(name)?;
        Ok(())
    }

    pub fn delete_branch(&mut self, name: &str) -> Result<()> {
        self.refs.delete_branch(name)?;
        Ok(())
    }

    pub fn switch_branch(&mut self, name: &str, force: bool) -> Result<()> {
        if self.has_uncommitted_changes() && !force {
            return Err(error::RepositoryError::UncommitedChanges);
        }

        self.refs.switch_branch(name)?;
        self.index.clear()?;
        self.work_tree.clear()?;
        let commit = self.refs.head_commit()?;

        if !commit.is_empty() {
            self.work_tree
                .restore_from_commit(&commit, &self.object_store)?
        }

        Ok(())
    }

    pub async fn push(&mut self, url: Option<String>) -> Result<()> {
        let url = match url {
            Some(u) => u,
            None => self
                .config
                .get("origin")
                .map_err(|_| error::RepositoryError::MissingOrigin())?,
        };

        self.config.set("origin".to_string(), url.clone())?;

        let mut client = GrpcClient::connect_remote(url)
            .await
            .map_err(|e| error::RepositoryError::from("Connection to remote failed.", e))?;
        let response = client
            .push()
            .await
            .map_err(|e| error::RepositoryError::from("Failed to push to remote", e))?;

        println!("Server response: {}", response.response_message);
        println!("Status code: {}", response.code);

        Ok(())
    }
}
