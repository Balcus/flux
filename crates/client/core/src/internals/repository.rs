use crate::error;
use crate::internals::config::{Config, Field};
use crate::internals::grpc_client::GrpcClient;
use crate::internals::index::Index;
use crate::internals::object_store::ObjectStore;
use crate::internals::refs::Refs;
use crate::internals::work_tree::WorkTree;
use crate::objects::blob::Blob;
use crate::objects::commit::Commit;
use crate::objects::object::Object;
use crate::objects::object_type::ObjectType;
use crate::utils::read_bytes_from_file;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tar::Archive;

pub type Result<T> = std::result::Result<T, error::RepositoryError>;

#[derive(Debug)]
pub struct Repository {
    pub name: String,
    pub refs: Refs,
    pub work_tree: WorkTree,
    pub flux_dir: PathBuf,
    pub config: Config,
    pub index: Index,
    pub object_store: ObjectStore,
}

//TODO:
/*
* Commands as classes for all supported commands
* Error handling for commands as classes
* Unit and integration tests for both client and server !!
* Add all supported commands to desktop app
* Object ids and functional shorthand for them
* File mode and executables
* Simplified cli interface
* Review branching
* Merging
*/
impl Repository {
    pub fn open(path: Option<String>) -> Result<Self> {
        let work_tree_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let work_tree_path = work_tree_path
            .canonicalize()
            .map_err(|e| error::IoError::metadata_error(&work_tree_path, e))?;

        let repo_name = work_tree_path
            .file_name()
            .ok_or_else(|| error::RepositoryError::PathName {
                path: work_tree_path.clone(),
            })?
            .to_string_lossy()
            .to_string();

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
            name: repo_name,
        })
    }

    pub async fn auth(&mut self, url: Option<String>) -> Result<()> {
        let url = match url {
            Some(u) => u,
            None => self
                .config
                .get_required(Field::Origin)
                .map_err(|_| error::RepositoryError::MissingOrigin())?,
        };
        self.config.set("origin".to_string(), url.clone())?;
        let mut client = GrpcClient::connect_remote(url).await?;
        let credentials = self.config.get_credentials()?;
        let token = client
            .auth(credentials.user_name, credentials.user_email)
            .await?;
        self.config
            .set("access_token".to_string(), token.access_token)?;
        Ok(())
    }

    pub async fn clone(url: String, path: Option<String>) -> Result<Self> {
        let mut clinet = GrpcClient::connect_remote(url).await?;
        let repo_name = clinet.repo_name()?;
        let archive = clinet.clone_repository().await?;
        let path = path.clone().unwrap_or(".".to_string());
        let repo_path = PathBuf::from(path).join(repo_name);
        let flux_dir = repo_path.join(".flux");
        Self::dearchive(archive, &flux_dir)?;
        let repository = Self::open(Some(repo_path.to_string_lossy().to_string()))?;
        repository.restore_fs()?;
        Ok(repository)
    }

    fn dearchive(archive_bytes: Vec<u8>, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir)?;
        let cursor = Cursor::new(archive_bytes);
        let decoder = GzDecoder::new(cursor);
        let mut archive = Archive::new(decoder);
        archive.unpack(output_dir)?;
        Ok(())
    }

    pub fn restore_fs(&self) -> Result<()> {
        let last_commit = self.refs.head_commit()?;
        self.work_tree
            .restore_from_commit(&last_commit, &self.object_store)?;
        Ok(())
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.config.set(key, value)?;
        Ok(())
    }

    pub fn add(&mut self, path: &str) -> Result<()> {
        let full_path = self.work_tree.path().join(path);
        self.add_path(&full_path)?;
        self.remove_deleted_files_from_index(path)?;
        Ok(())
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
        let data = read_bytes_from_file(path)?;
        let blob = Blob::from_bytes(data);
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

        self.index.add(rel_str.to_owned(), blob.id())?;

        Ok(())
    }

    fn remove_deleted_files_from_index(&mut self, path: &str) -> Result<()> {
        let full_path = self.work_tree.path().join(path);
        let metadata =
            fs::metadata(&full_path).map_err(|e| error::IoError::metadata_error(&full_path, e))?;

        if metadata.is_dir() {
            let prefix = if path == "." {
                String::new()
            } else {
                format!("{}/", path.trim_end_matches('/'))
            };

            let indexed_files: Vec<String> = self
                .index
                .map
                .keys()
                .filter(|k| prefix.is_empty() || k.starts_with(&prefix))
                .cloned()
                .collect();

            for indexed_path in indexed_files {
                let file_full_path = self.work_tree.path().join(&indexed_path);
                if !file_full_path.exists() {
                    self.index.remove(&indexed_path)?;
                    println!("Removed deleted file from index: {}", indexed_path);
                }
            }
        }

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

        let credentials = self
            .config
            .get_credentials()
            .map_err(error::RepositoryError::Credentials)?;
        let user_name = credentials.user_name;
        let user_email = credentials.user_email;

        let last = self.refs.head_commit()?;
        let parent = (!last.is_empty()).then_some(last);
        let commit = Commit::new(tree_hash, user_name, user_email, parent, message);
        self.object_store.store(&commit)?;
        let hash = commit.id();
        self.refs.update_head(&hash)?;

        Ok(hash.to_string())
    }

    pub fn log(&self, _reference: Option<String>) -> Result<()> {
        let mut current_hash = self.refs.head_commit().ok().filter(|s| !s.is_empty());

        while let Some(hash) = current_hash {
            let obj = self.object_store.retrieve_object(&hash)?;
            println!("{}", obj);
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
        if !self.refs.exists(name) {
            return Err(error::RepositoryError::Refs(
                error::RefsError::MissingBranch(name.to_string()),
            ));
        }

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
        let content = self.archive()?;
        let credentials = self.config.get_credentials()?;

        let access_token = credentials
            .access_token
            .ok_or_else(|| error::RepositoryError::MissingToken)?;

        let url = match url {
            Some(u) => u,
            None => self
                .config
                .get_required(Field::Origin)
                .map_err(|_| error::RepositoryError::MissingOrigin())?,
        };

        let mut client = GrpcClient::connect_remote(url.clone())
            .await
            .map_err(|e| error::RepositoryError::from("Connection to remote failed.", e))?;

        let response = client
            .push(
                self.name.clone(),
                content,
                credentials.user_email,
                credentials.user_name,
                access_token,
            )
            .await
            .map_err(|e| error::RepositoryError::from("Failed to push to remote", e))?;

        self.config.set("origin".to_string(), url)?;
        println!("Server response: {}", response.response_message);

        Ok(())
    }

    fn archive(&self) -> Result<Vec<u8>> {
        let flux_dir = self
            .flux_dir
            .canonicalize()
            .map_err(error::RepositoryError::Archive)?;

        let mut buf: Vec<u8> = Vec::new();
        let gz = GzEncoder::new(&mut buf, Compression::default());
        let mut tar = tar::Builder::new(gz);

        for entry in fs::read_dir(&flux_dir).map_err(error::RepositoryError::Archive)? {
            let entry = entry.map_err(error::RepositoryError::Archive)?;
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if file_name == "config" {
                let mut header = tar::Header::new_gnu();
                header
                    .set_path("config")
                    .map_err(error::RepositoryError::Archive)?;
                header.set_size(0);
                header.set_mode(0o644);
                header.set_cksum();
                tar.append(&header, std::io::empty())
                    .map_err(error::RepositoryError::Archive)?;
            } else if path.is_file() {
                tar.append_path_with_name(&path, file_name)
                    .map_err(error::RepositoryError::Archive)?;
            } else if path.is_dir() {
                tar.append_dir_all(file_name, &path)
                    .map_err(error::RepositoryError::Archive)?;
            }
        }

        tar.into_inner()
            .map_err(error::RepositoryError::Archive)?
            .finish()
            .map_err(error::RepositoryError::Archive)?;

        Ok(buf)
    }

    pub fn cat(&self, hash: &str) -> Result<()> {
        let obj = self.object_store.retrieve_object(hash)?;
        println!("{}", obj);

        Ok(())
    }

    pub fn commit_tree(
        &self,
        tree_hash: String,
        message: String,
        parent_hash: Option<String>,
    ) -> Result<String> {
        let credentials = self
            .config
            .get_credentials()
            .map_err(error::RepositoryError::Credentials)?;

        let user_name = credentials.user_name;
        let user_email = credentials.user_email;

        let tree = self.object_store.retrieve_object(&tree_hash)?;

        if tree.object_type() != ObjectType::Tree {
            return Err(error::RepositoryError::CommitRoot { hash: tree.id() });
        }

        let commit = Commit::new(tree.id(), user_name, user_email, parent_hash, message);
        self.object_store.store(&commit)?;
        Ok(commit.id())
    }

    fn has_uncommitted_changes(&self) -> bool {
        let Ok(head_commit_hash) = self.refs.head_commit() else {
            return false;
        };

        let commit_obj = self
            .object_store
            .retrieve_object(&head_commit_hash)
            .unwrap();
        let commit = commit_obj.as_any().downcast_ref::<Commit>().unwrap();
        let head_tree_hash = &commit.tree_hash;

        let current_tree_hash = self
            .work_tree
            .build_tree_from_index(&self.index.map, &self.object_store)
            .unwrap();

        head_tree_hash != &current_tree_hash
    }
}
