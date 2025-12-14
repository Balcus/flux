use crate::objects::blob::Blob;
use crate::repo::config::Config;
use crate::repo::object_type::ObjectType;
use anyhow::{Context, bail};
use flate2::read::ZlibDecoder;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct Repository {
    pub work_tree: PathBuf,
    pub git_dir: PathBuf,
    pub config: Config,
}

impl Repository {
    pub fn init<P: AsRef<Path>>(path: Option<P>) -> Result<(), anyhow::Error> {
        let work_tree = match &path {
            Some(p) => p.as_ref().to_path_buf(),
            None => PathBuf::from("."),
        };

        let git_dir = work_tree.join(".git");

        if git_dir.exists() {
            println!("Repository already initialized");
            return Ok(());
        }

        fs::create_dir(&git_dir)?;

        let config_path = &git_dir.join("config");
        let _ = Config::default(config_path);

        fs::create_dir(&git_dir.join("objects"))?;
        fs::create_dir(&git_dir.join("refs"))?;
        fs::write(&git_dir.join("HEAD"), "ref: refs/heads/main\n")?;

        println!("Initialized repository");

        Ok(())
    }

    pub fn open(path: Option<impl AsRef<Path>>) -> Result<Self, anyhow::Error> {
        let work_tree = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => PathBuf::from("."),
        };

        let git_dir = work_tree.join(".git");

        if !git_dir.exists() {
            bail!("Not a git repository");
        }

        let config_path = git_dir.join("config");

        Ok(Self {
            config: Config::from(&config_path)?,
            work_tree,
            git_dir,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<(), anyhow::Error> {
        self.config.set(key, value)?;
        Ok(())
    }

    fn read_object(&self, hash: String) -> Result<(ObjectType, usize, Vec<u8>), anyhow::Error> {
        let (dir, file) = hash.split_at(2);
        let path = PathBuf::from(format!(
            "{}/.git/objects/{}/{}",
            self.work_tree.display(),
            dir,
            file
        ));

        let compressed = fs::read(path)
            .with_context(|| format!("Could not read git object for hash: {}", &hash))?;

        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut byte = [0u8; 1];
        let mut header = Vec::new();

        loop {
            decoder.read_exact(&mut byte)?;
            if byte[0] == b'\0' {
                break;
            }
            header.push(byte[0]);

            if header.len() > 100 {
                return Err(anyhow::anyhow!("Invalid git object header"));
            }
        }

        let header_str =
            String::from_utf8(header).with_context(|| "Could not parse git object header")?;
        let header_parts: Vec<&str> = header_str.split(' ').collect();
        if header_parts.len() != 2 {
            anyhow::bail!("Invalid git object header");
        }

        let object_type = match header_parts[0] {
            "blob" => ObjectType::Blob,
            "tree" => ObjectType::Tree,
            "commit" => ObjectType::Commit,
            _ => anyhow::bail!("Unknown git object type"),
        };
        let size: usize = header_parts[1].parse()?;
        let mut content = vec![0u8; size];
        decoder.read_exact(&mut content)?;

        Ok((object_type, size, content))
    }

    pub fn cat_file(&self, hash: String) -> Result<(), anyhow::Error> {
        let (object_type, size, content) = self.read_object(hash)?;
        let output: String;
        match object_type {
            ObjectType::Blob => {
                output = Blob::cat_file(size, content)?;
            }
            _ => {
                anyhow::bail!("cat_file currently supports only blob objects");
            }
        }
        println!("{}", output);
        Ok(())
    }

    pub fn hash_object(&self, path: String, write: bool) -> Result<String, anyhow::Error> {
        let full_path = self.work_tree.join(&path);
        let metadata = fs::metadata(&full_path)?;
        let hash: String;

        match metadata.is_file() {
            true => {
                let content = fs::read(&full_path)
                    .with_context(|| format!("Could not read file {:?}", full_path))?;
                let (h, compressed_file) = Blob::hash_object(content)?;
                hash = h;
                if write {
                    let (dir, file) = hash.split_at(2);
                    let object_path = self
                        .work_tree
                        .join(format!(".git/objects/{}/{}", dir, file));
                    fs::create_dir_all(self.work_tree.join(format!(".git/objects/{}", dir)))?;
                    fs::write(object_path, &compressed_file)?;
                }
            }
            false if metadata.is_dir() => {
                hash = self
                    .tree_from_dir(&full_path)
                    .with_context(|| "Failed creating tree object")?;
            }
            _ => {
                anyhow::bail!("Unsupported file type for hashing");
            }
        }

        Ok(hash)
    }

    pub fn tree_from_dir(&self, path: &Path) -> Result<String, anyhow::Error> {
        let mut entries: Vec<(String, String, String)> = Vec::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let metadata = fs::metadata(&entry_path)?;

            if metadata.is_file() {
                let file_name = entry.file_name().into_string().unwrap();
                let relative_path = entry_path
                    .strip_prefix(&self.work_tree)
                    .with_context(|| "Failed to get relative path")?
                    .to_string_lossy()
                    .to_string();
                let blob_hash = self.hash_object(relative_path, true)?;
                entries.push(("100644".to_string(), blob_hash, file_name));
            } else if metadata.is_dir() {
                let dir_name = entry.file_name().into_string().unwrap();
                let tree_hash = self.tree_from_dir(&entry_path)?;
                entries.push(("40000".to_string(), tree_hash, dir_name));
            }
        }

        let mut tree_content: Vec<u8> = Vec::new();
        for (mode, hash, name) in entries {
            let hash_bytes = hex::decode(hash).unwrap();
            let entry = format!("{} {}\0", mode, name);
            tree_content.extend_from_slice(entry.as_bytes());
            tree_content.extend_from_slice(&hash_bytes);
        }

        let (tree_hash, compressed) = crate::objects::tree::Tree::hash_object(tree_content)?;
        let dest = PathBuf::from(format!(
            "{}/.git/objects/{}/{}",
            self.work_tree.display(),
            &tree_hash[0..2],
            &tree_hash[2..]
        ));
        fs::create_dir_all(dest.parent().unwrap())?;
        fs::write(dest, &compressed)?;

        Ok(tree_hash)
    }
}
