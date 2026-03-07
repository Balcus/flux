use crate::{
    commands::command::Command, internals::repository::Repository, objects::{blob::Blob, object::Object, tree::Tree}, utils::read_bytes_from_file
};
use anyhow::Context;
use std::{env, fs, path::PathBuf};

pub struct HashObject<'a> {
    pub path: &'a str,
    pub write: bool,
}

impl<'a> HashObject<'a> {
    pub fn new(path: &'a str, write: bool) -> Self {
        Self { path, write }
    }

    pub fn hash(&mut self, repo_path: Option<String>) -> anyhow::Result<String> {
        let full_path = PathBuf::from(&self.path);
        let full_path = if full_path.is_absolute() {
            full_path
        } else {
            env::current_dir()?.join(self.path)
        };

        let metadata = fs::metadata(&full_path)
            .with_context(|| format!("path does not exist: {}", full_path.display()))?;

        let object: Box<dyn Object> = if metadata.is_file() {
            let data = read_bytes_from_file(&full_path)?;
            Box::new(Blob::from_bytes(data))
        } else {
            Box::new(Tree::new(&full_path).unwrap())
        };

        if self.write {
            let repo = Repository::open(repo_path)?;
            repo.object_store.store(object.as_ref())?;
        }

        Ok(object.id())
    }
}

impl<'a> Command for HashObject<'a> {
    fn run(&mut self) -> anyhow::Result<()> {
        let hash = self.hash(None)?;
        println!("{}", hash);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::command::Command;
    use crate::commands::hash_object::HashObject;
    use crate::commands::init::InitCommand;
    use anyhow::Context;
    use std::fs;
    use tempfile::tempdir;

    fn git_hash_object(path: &str) -> anyhow::Result<String> {
        let output = std::process::Command::new("git")
            .args(["hash-object", "--no-filters", path])
            .output()
            .context("Failed to execute git hash-object")?;
        if !output.status.success() {
            anyhow::bail!(
                "git hash-object failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    #[test]
    pub fn hash_file_no_write() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.txt");
        let file_path_string = file_path.to_string_lossy().to_string();
        fs::write(&file_path, "hello world")?;

        let flux_hash = HashObject::new(&file_path_string, false).hash(None)?;
        let git_hash = git_hash_object(&file_path_string)?;
        assert_eq!(flux_hash, git_hash);
        assert!(!dir.path().join(".flux/objects").exists());

        Ok(())
    }

    #[test]
    pub fn hash_dir_no_write() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let src = dir.path().join("src");
        fs::create_dir(&src)?;
        fs::write(src.join("main.rs"), "fn main() {}")?;
        fs::write(src.join("lib.rs"), "pub fn add() {}")?;
        let src_string = src.to_string_lossy().to_string();

        let flux_hash = HashObject::new(&src_string, false).hash(None)?;
        assert_eq!(flux_hash.len(), 40);
        assert!(flux_hash.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(!dir.path().join(".flux/objects").exists());

        Ok(())
    }

    #[test]
    pub fn hash_file_write() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), false).run()?;

        let file_path = dir.path().join("test.txt");
        let file_path_string = file_path.to_string_lossy().to_string();
        fs::write(&file_path, "hello world")?;

        let flux_hash = HashObject::new(&file_path_string, true).hash(Some(repo_path))?;
        let git_hash = git_hash_object(&file_path_string)?;
        assert_eq!(flux_hash, git_hash);

        let object_path = dir
            .path()
            .join(".flux/objects")
            .join(&flux_hash[..2])
            .join(&flux_hash[2..]);
        assert!(object_path.exists());

        Ok(())
    }

    #[test]
    pub fn hash_dir_write() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), false).run()?;

        let src = dir.path().join("src");
        fs::create_dir(&src)?;
        fs::write(src.join("main.rs"), "fn main() {}")?;
        fs::write(src.join("lib.rs"), "pub fn add() {}")?;
        let src_string = src.to_string_lossy().to_string();

        let flux_hash = HashObject::new(&src_string, true).hash(Some(repo_path))?;
        assert_eq!(flux_hash.len(), 40);
        assert!(flux_hash.chars().all(|c| c.is_ascii_hexdigit()));

        let object_path = dir
            .path()
            .join(".flux/objects")
            .join(&flux_hash[..2])
            .join(&flux_hash[2..]);
        assert!(object_path.exists());

        Ok(())
    }
}
