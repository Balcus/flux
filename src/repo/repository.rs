use crate::objects::blob;
use crate::objects::commit;
use crate::objects::tree;
use crate::repo::config::Config;
use crate::shared::types::object_type::ObjectType;
use crate::utils;
use anyhow::{Context, bail};
use std::fs;
use std::path::PathBuf;

pub struct Repository {
    pub work_tree: PathBuf,
    pub git_dir: PathBuf,
    pub config: Config,
}

impl Repository {
    pub fn init(path: Option<String>, force: bool) -> anyhow::Result<Self> {
        let work_tree = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let git_dir = work_tree.join(".git");

        if git_dir.join("config").exists() && !force {
            bail!("Repository already initialized");
        }

        fs::create_dir_all(&git_dir)?;
        fs::create_dir(git_dir.join("objects"))?;
        fs::create_dir(git_dir.join("refs"))?;
        let config = Config::default(&git_dir.join("config"))?;
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;
        println!("Initialized repository");

        Ok(Self {
            work_tree,
            git_dir,
            config,
        })
    }

    pub fn open(path: Option<String>) -> anyhow::Result<Self> {
        let work_tree = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let git_dir = work_tree.join(".git");

        if !git_dir.exists() {
            bail!("Not a git repository");
        }

        let config_path = git_dir.join("config");
        let config = Config::from(&config_path)?;

        Ok(Self {
            config,
            work_tree,
            git_dir,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<(), anyhow::Error> {
        self.config.set(key, value)?;
        Ok(())
    }

    pub fn hash_object(&self, path: String, write: bool) -> anyhow::Result<String> {
        let full_path = self.work_tree.join(&path);
        let metadata = fs::metadata(&full_path).context("Failed to read file metadata")?;

        let result = if metadata.is_file() {
            let content = fs::read(&full_path)?;
            let blob = blob::hash_blob(content)?;

            if write {
                utils::write_object(&self.git_dir, &blob.object_hash, &blob.compressed_content)?;
            }

            blob
        } else if metadata.is_dir() {
            let builder = tree::TreeBuilder {
                work_tree: &self.work_tree,
                git_dir: &self.git_dir,
            };

            builder.write_tree(&full_path)?
        } else {
            bail!("Unsupported file type");
        };

        Ok(result.object_hash)
    }

    pub fn cat_file(&self, object_hash: String) -> anyhow::Result<()> {
        let object = utils::read_object(&self.git_dir, &object_hash)?;

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
                commit::show_commit(&self.git_dir, object_hash)?;
            }
            _ => bail!("cat_file currently supports only blob objects"),
        }

        Ok(())
    }

    pub fn ls_tree(&self, tree_hash: String) -> anyhow::Result<()> {
        let object = utils::read_object(&self.git_dir, &tree_hash)?;

        match object.object_type {
            ObjectType::Tree => {
                let mut result: String = String::new();
                let mut i = 0;
                let content = object.decompressed_content;

                while i < content.len() {
                    let mode_end = content[i..].iter().position(|&b| b == b' ').unwrap();
                    let mode = std::str::from_utf8(&content[i..i + mode_end])?;
                    i += mode_end + 1;

                    let name_end = content[i..].iter().position(|&b| b == b'\0').unwrap();
                    let name = std::str::from_utf8(&content[i..i + name_end])?;
                    i += name_end + 1;

                    let hash = hex::encode(&content[i..i + 20]);
                    i += 20;

                    let object_type = if mode.starts_with("040") {
                        "tree"
                    } else {
                        "blob"
                    };

                    result.push_str(&format!("{mode} {object_type} {hash} {name}\n"));
                }

                print!("{}", result);
            }
            _ => bail!("ls_tree requires a tree object"),
        }
        Ok(())
    }

    pub fn commit_tree(&self, tree_hash: String, message: String) -> anyhow::Result<String> {
        let user_name =
            self.config.user_name.clone().context(
                "Please configure user settings (user_name) in order to create a commit",
            )?;

        let user_email =
            self.config.user_email.clone().context(
                "Please configure user settings (user_email) in order to create a commit",
            )?;

        let object = utils::read_object(&self.git_dir, &tree_hash)?;

        let hash = match object.object_type {
            ObjectType::Tree => commit::commit_tree(
                &self.git_dir,
                user_name,
                user_email,
                tree_hash,
                None,
                message,
            )?,
            _ => bail!("Can only commit tree objects"),
        };
        Ok(hash)
    }
}
