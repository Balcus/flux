use crate::database::commit::Commit;
use crate::database::database::Database;
use crate::database::object::Object;
use crate::database::object_type::ObjectType;
use crate::dircache::index::Index;
use crate::internals::config::{Config, Field};
use crate::internals::grpc_client::GrpcClient;
use crate::internals::refs::Refs;
use crate::internals::work_tree::WorkTree;
use crate::status::status_impl::Status;
use anyhow::{Context, bail};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tar::Archive;

#[derive(Debug)]
pub struct Repository {
    pub name: String,
    pub refs: Refs,
    pub work_tree: WorkTree,
    pub flux_dir: PathBuf,
    pub config: Config,
}

impl Repository {
    pub fn open(path: Option<String>) -> anyhow::Result<Self> {
        let work_tree_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let work_tree_path = work_tree_path.canonicalize().with_context(|| {
            format!(
                "Failed to read metadata for '{}'.",
                work_tree_path.display()
            )
        })?;

        let repo_name = work_tree_path
            .file_name()
            .with_context(|| format!("Failed to operate on path: '{}'.", work_tree_path.display()))?
            .to_string_lossy()
            .to_string();

        let store_dir = work_tree_path.join(".flux");

        if !store_dir.exists() {
            bail!(
                "Repository not initialized at: '{}'. Run 'flux init' and try again.",
                work_tree_path.display()
            );
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

    pub async fn auth(&mut self, url: Option<String>) -> anyhow::Result<()> {
        let url = match url {
            Some(u) => u,
            None => self.config.get_required(Field::Origin)
                .context("Missing origin for remote repository. Specify it with 'flux push http://originurl' or set it with 'flux set origin http://originurl'.")?,
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

    pub async fn clone(url: String, path: Option<String>) -> anyhow::Result<Self> {
        let mut client = GrpcClient::connect_remote(url).await?;
        let repo_name = client.repo_name()?;
        let archive = client.clone_repository().await?;
        let path = path.unwrap_or(".".to_string());
        let repo_path = PathBuf::from(path).join(repo_name);
        let flux_dir = repo_path.join(".flux");
        Self::dearchive(archive, &flux_dir)?;
        let repository = Self::open(Some(repo_path.to_string_lossy().to_string()))?;
        repository.restore_fs()?;
        Ok(repository)
    }

    fn dearchive(archive_bytes: Vec<u8>, output_dir: &Path) -> anyhow::Result<()> {
        fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create '{}'.", output_dir.display()))?;
        let cursor = Cursor::new(archive_bytes);
        let decoder = GzDecoder::new(cursor);
        let mut archive = Archive::new(decoder);
        archive
            .unpack(output_dir)
            .with_context(|| format!("Failed to unpack archive to '{}'.", output_dir.display()))?;
        Ok(())
    }

    pub fn restore_fs(&self) -> anyhow::Result<()> {
        let last_commit = self.refs.head_commit()?;
        self.work_tree.restore_from_commit(&last_commit)?;
        Ok(())
    }

    pub fn set(&mut self, key: String, value: String) -> anyhow::Result<()> {
        self.config.set(key, value)?;
        Ok(())
    }

    pub fn show_branches(&self) -> anyhow::Result<String> {
        Ok(self.refs.format_branches()?)
    }

    pub fn list_branches(&self) -> anyhow::Result<Vec<String>> {
        Ok(self.refs.list_branches()?)
    }

    pub fn new_branch(&mut self, name: &str) -> anyhow::Result<()> {
        self.refs.new_branch(name)?;
        Ok(())
    }

    pub fn delete_branch(&mut self, name: &str) -> anyhow::Result<()> {
        self.refs.delete_branch(name)?;
        Ok(())
    }

    pub async fn push(&mut self, url: Option<String>) -> anyhow::Result<()> {
        let content = self.archive()?;
        let credentials = self.config.get_credentials()?;

        let access_token = credentials.access_token.context(
            "Missing access token from remote server. Try running flux auth and try again.",
        )?;

        let url = match url {
            Some(u) => u,
            None => self.config.get_required(Field::Origin)
                .context("Missing origin for remote repository. Specify it with 'flux push http://originurl' or set it with 'flux set origin http://originurl'.")?,
        };

        let mut client = GrpcClient::connect_remote(url.clone()).await?;
        let response = client
            .push(
                self.name.clone(),
                content,
                credentials.user_email,
                credentials.user_name,
                access_token,
            )
            .await?;

        self.config.set("origin".to_string(), url)?;
        println!("Server response: {}", response.response_message);
        Ok(())
    }

    fn archive(&self) -> anyhow::Result<Vec<u8>> {
        let flux_dir = self
            .flux_dir
            .canonicalize()
            .with_context(|| format!("Failed to read '{}'.", self.flux_dir.display()))?;

        let mut buf: Vec<u8> = Vec::new();
        let gz = GzEncoder::new(&mut buf, Compression::default());
        let mut tar = tar::Builder::new(gz);

        for entry in fs::read_dir(&flux_dir)
            .with_context(|| format!("Failed to read '{}'.", flux_dir.display()))?
        {
            let entry =
                entry.with_context(|| format!("Failed to read '{}'.", flux_dir.display()))?;
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if file_name == "config" {
                let mut header = tar::Header::new_gnu();
                header
                    .set_path("config")
                    .context("Failed to archive flux repository.")?;
                header.set_size(0);
                header.set_mode(0o644);
                header.set_cksum();
                tar.append(&header, std::io::empty())
                    .context("Failed to archive flux repository.")?;
            } else if path.is_file() {
                tar.append_path_with_name(&path, file_name)
                    .context("Failed to archive flux repository.")?;
            } else if path.is_dir() {
                tar.append_dir_all(file_name, &path)
                    .context("Failed to archive flux repository.")?;
            }
        }

        tar.into_inner()
            .context("Failed to archive flux repository.")?
            .finish()
            .context("Failed to archive flux repository.")?;

        Ok(buf)
    }

    pub fn cat(&self, hash: &str) -> anyhow::Result<()> {
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
    ) -> anyhow::Result<String> {
        let credentials = self.config.get_credentials()?;
        let db = Database::open(self.flux_dir.clone());
        let tree = db.read_object(&tree_hash).unwrap();

        if tree.object_type() != ObjectType::Tree {
            bail!(
                "Cannot use object {} as commit root, object is not a tree.",
                tree.id()
            );
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

    fn load_index(&self) -> anyhow::Result<Index> {
        let mut index = Index::new(self.flux_dir.join("index"));
        index.load()?;
        Ok(index)
    }

    fn load_status(&self) -> anyhow::Result<Status> {
        Ok(Status::new(&self)?)
    }

    pub fn switch_branch(&mut self, name: &str, force: bool) -> anyhow::Result<()> {
        if !self.refs.exists(name) {
            bail!("Missing target branch: '{}'.", name);
        }

        if !force {
            let status = self.load_status()?;
            if status.has_staged_changes() || status.has_unstaged_changes() || status.has_untracked_files() {
                bail!(
                    "Switching branches would overwrite the current changes. Please discard them or use the force flag"
                );
            }
        }

        self.refs.switch_branch(name)?;
        self.refs = Refs::load(&self.flux_dir)?;
        self.work_tree.clear()?;

        let mut index = self.load_index()?;
        index.entries.clear();

        let commit = self.refs.head_commit()?;
        if !commit.is_empty() {
            self.work_tree.restore_from_commit(&commit)?;

            let db = Database::open(self.flux_dir.clone());
            let file_map = db.commit_to_map(commit)?;

            for (path, hash) in file_map {
                let full_path = self.work_tree.path().join(&path);
                let stat = fs::metadata(&full_path)?;
                let id_bytes = hex::decode(&hash)?;
                let id: [u8; 20] = id_bytes
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid SHA"))?;
                let entry =
                    crate::dircache::index_entry::IndexEntry::create(path.clone(), id, &stat, 0);
                index.entries.insert((path, 0), entry);
            }
        }

        index.mark_changed();
        index.write_updates()?;
        Ok(())
    }
}
