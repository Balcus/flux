use crate::database::commit::Commit;
use crate::database::database::Database;
use crate::database::object::Object;
use crate::database::object_type::ObjectType;
use crate::dircache::index::Index;
use crate::error;
use crate::internals::config::{Config, Field};
use crate::internals::grpc_client::GrpcClient;
use crate::internals::refs::Refs;
use crate::internals::work_tree::WorkTree;
use crate::status::status_impl::Status;
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
}

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
        let refs = Refs::load(&store_dir)?;

        Ok(Self {
            refs,
            work_tree: WorkTree::new(work_tree_path),
            flux_dir: store_dir,
            config,
            name: repo_name,
        })
    }

    fn load_index(&self) -> anyhow::Result<Index> {
        let mut index = Index::new(self.flux_dir.join("index"));
        index.load()?;
        Ok(index)
    }

    fn load_status(&self) -> anyhow::Result<Status> {
        let status = Status::new(&self)?;
        Ok(status)
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
        let mut client = GrpcClient::connect_remote(url).await?;
        let repo_name = client.repo_name()?;
        let archive = client.clone_repository().await?;
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
        self.work_tree.restore_from_commit(&last_commit)?;
        Ok(())
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.config.set(key, value)?;
        Ok(())
    }

    pub fn commit(&mut self, message: String) -> anyhow::Result<String> {
        let index = self.load_index()?;
        let status = self.load_status()?;

        if status.is_clean() {
            anyhow::bail!("Working tree clean, nothing to commit.");
        }

        let db = Database::open(self.flux_dir.clone());

        let tree_hash = self
            .work_tree
            .build_tree_from_index(&index, &db)
            .unwrap();

        let credentials = self
            .config
            .get_credentials()
            .map_err(error::RepositoryError::Credentials)?;

        let last = self.refs.head_commit()?;
        let parent = (!last.is_empty()).then_some(last);

        let commit = Commit::new(
            tree_hash,
            credentials.user_name,
            credentials.user_email,
            parent,
            message,
        );

        let hash = commit.id();
        db.store(Box::new(commit)).unwrap();
        self.refs.update_head(&hash)?;

        Ok(hash)
    }

    pub fn log(&self, _reference: Option<String>) -> Result<()> {
        let db = Database::open(self.flux_dir.clone());
        let mut current_hash = self.refs.head_commit().ok().filter(|s| !s.is_empty());

        while let Some(hash) = current_hash {
            let obj = db.read_object(&hash).unwrap();
            println!("{}", obj);
            let current = db.read_object(&hash).unwrap();
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
        let db = Database::open(self.flux_dir.clone());
        let obj = db.read_object(hash).unwrap();
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

        let db = Database::open(self.flux_dir.clone());
        let tree = db.read_object(&tree_hash).unwrap();

        if tree.object_type() != ObjectType::Tree {
            return Err(error::RepositoryError::CommitRoot { hash: tree.id() });
        }

        let commit = Commit::new(
            tree.id(),
            credentials.user_name,
            credentials.user_email,
            parent_hash,
            message,
        );
        db.store(Box::new(commit.clone())).unwrap();
        Ok(commit.id())
    }

    pub fn switch_branch(&mut self, name: &str, force: bool) -> anyhow::Result<()> {
        if !self.refs.exists(name) {
            anyhow::bail!("Missing target branch: {}.", name);
        }

        if !force {
            let status = self.load_status()?;
            if status.has_staged_changes() {
                anyhow::bail!(
                    "There are still uncommited changes, use --force if you are sure about what you are doing."
                )
            }
        }

        self.refs.switch_branch(name)?;

        let mut index = self.load_index()?;
        index.entries.clear();
        index.write_updates()?;

        self.work_tree.clear()?;
        let commit = self.refs.head_commit()?;
        if !commit.is_empty() {
            self.work_tree.restore_from_commit(&commit)?;
        }

        Ok(())
    }
}
