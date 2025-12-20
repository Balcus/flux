use crate::repo::config::Config;
use crate::repo::utils::{self, HashResult, TreeEntry};
use anyhow::{Context, bail};
use std::fs;
use std::path::{Path, PathBuf};

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
            utils::hash_blob(content)?
        } else if metadata.is_dir() {
            self.hash_tree_recursive(&full_path)?
        } else {
            bail!("Unsupported file type");
        };

        if write {
            utils::write_object(
                &self.git_dir,
                &result.object_hash,
                &result.compressed_content,
            )?;
        }

        Ok(result.object_hash)
    }

    fn hash_tree_recursive(&self, path: &Path) -> anyhow::Result<HashResult> {
        let entries = self.collect_tree_entries(path)?;
        let tree_content = utils::build_tree_content(entries);
        utils::hash_tree(tree_content)
    }

    fn collect_tree_entries(&self, path: &Path) -> anyhow::Result<Vec<TreeEntry>> {
        let mut entries = Vec::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                continue;
            }

            let metadata = fs::metadata(&entry_path)?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("Unsupported file name"))?;

            if metadata.is_file() {
                let relative_path = entry_path
                    .strip_prefix(&self.work_tree)?
                    .to_string_lossy()
                    .to_string();

                let hash = self.hash_object(relative_path, true)?;
                entries.push(TreeEntry {
                    mode: "100644".to_string(),
                    entry_type: "blob".into(),
                    hash,
                    name,
                });
            } else if metadata.is_dir() {
                let result = self.hash_tree_recursive(&entry_path)?;

                utils::write_object(
                    &self.git_dir,
                    &result.object_hash,
                    &result.compressed_content,
                )?;

                entries.push(TreeEntry {
                    mode: "040000".to_string(),
                    entry_type: "tree".into(),
                    hash: result.object_hash,
                    name,
                });
            }
        }

        Ok(entries)
    }

    pub fn cat_file(&self, object_hash: String) -> anyhow::Result<()> {
        let object = utils::read_object(&self.git_dir, &object_hash)?;

        match object.object_type {
            utils::ObjectType::Blob => {
                let output =
                    String::from_utf8(object.content).context("Blob contains invalid UTF-8")?;
                print!("{}", output);
            }
            _ => bail!("cat_file currently supports only blob objects"),
        }

        Ok(())
    }

    pub fn ls_tree(&self, tree_hash: String) -> anyhow::Result<()> {
        let object = utils::read_object(&self.git_dir, &tree_hash)?;

        match object.object_type {
            utils::ObjectType::Tree => {
                let mut result: String = String::new();
                let mut i = 0;
                let content = object.content;

                while i < content.len() {
                    let mode_end = content[i..].iter().position(|&b| b == b' ').unwrap();
                    let mode = std::str::from_utf8(&content[i..i + mode_end])?;
                    i += mode_end + 1;

                    let type_end = content[i..].iter().position(|&b| b == b' ').unwrap();
                    let object_type = std::str::from_utf8(&content[i..i + type_end])?;
                    i += type_end + 1;

                    let name_end = content[i..].iter().position(|&b| b == b'\0').unwrap();
                    let name = std::str::from_utf8(&content[i..i + name_end])?;
                    i += name_end + 1;

                    let hash = hex::encode(&content[i..i + 20]);
                    i += 20;

                    result.push_str(&format!("{mode} {object_type} {name} {hash}\n"));
                }
                print!("{}", result);
            }
            _ => bail!("ls_tree requires a tree object"),
        }

        Ok(())
    }
}
