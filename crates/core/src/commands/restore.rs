use crate::{
    commands::command::Command,
    database::{database::Database, walker::Walker},
    dircache::index::Index,
    internals::repository::Repository,
};

pub struct RestoreCommand<'a> {
    pub repo: &'a mut Repository,
    pub path: String,
}

impl<'a> RestoreCommand<'a> {
    pub fn new(repo: &'a mut Repository, path: String) -> Self {
        Self { repo, path }
    }
}

impl<'a> Command for RestoreCommand<'a> {
    type Output = ();
    
    fn run(&mut self) -> anyhow::Result<()> {
        let mut index = Index::new(self.repo.flux_dir.join("index"));
        index.load()?;

        let head_commit = self.repo.refs.head_commit()?;
        if head_commit.is_empty() {
            index.rm(self.path.clone())?;
            index.write_updates()?;
            return Ok(());
        }

        let db = Database::open(self.repo.flux_dir.clone());
        let walker = Walker::new(&db);

        match walker.file_hash_from_commit(&self.path, &head_commit)? {
            Some(blob_hash) => {
                let abs_path = self.repo.work_tree.path().join(&self.path);
                let stat = if abs_path.exists() {
                    std::fs::metadata(&abs_path)?
                } else {
                    let parent = abs_path
                        .parent()
                        .ok_or_else(|| anyhow::anyhow!("Path has no parent"))?;
                    std::fs::metadata(parent)?
                };
                index.add(self.path.clone(), blob_hash, stat)?;
            }
            None => {
                if index.entries.contains_key(&(self.path.clone(), 0)) {
                    index.rm(self.path.clone())?;
                } else {
                    anyhow::bail!("'{}' is not tracked", self.path);
                }
            }
        }

        index.write_updates()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        commands::{
            add::AddCommand, command::Command, commit::CommitCommand, init::InitCommand,
            restore::RestoreCommand,
        },
        dircache::index::Index,
        internals::repository::Repository,
    };
    use std::{fs, path::PathBuf};
    use tempfile::tempdir;

    fn setup_repo_with_file(
        content: &str,
        filename: &str,
    ) -> anyhow::Result<(tempfile::TempDir, Repository)> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;
        repo.set("user_name".to_string(), "test".to_string())?;
        repo.set("user_email".to_string(), "test@test.com".to_string())?;
        fs::write(dir.path().join(filename), content)?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;
        CommitCommand::new(&mut repo, "initial commit".to_string())?.run()?;
        Ok((dir, repo))
    }

    #[test]
    fn restore_staged_modification() -> anyhow::Result<()> {
        let (dir, mut repo) = setup_repo_with_file("original", "file.txt")?;
        let file_path = dir.path().join("file.txt");

        fs::write(&file_path, "modified")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;

        let mut index = Index::new(repo.flux_dir.join("index"));
        index.load()?;
        let entry = index.entries.get(&("file.txt".to_string(), 0)).unwrap();
        let staged_id = hex::encode(entry.id);

        RestoreCommand::new(&mut repo, "file.txt".to_string()).run()?;

        let mut index = Index::new(repo.flux_dir.join("index"));
        index.load()?;
        let entry = index.entries.get(&("file.txt".to_string(), 0)).unwrap();
        let restored_id = hex::encode(entry.id);

        assert_ne!(staged_id, restored_id);
        assert_eq!(fs::read_to_string(&file_path)?, "modified");

        Ok(())
    }

    #[test]
    fn restore_does_not_touch_disk() -> anyhow::Result<()> {
        let (dir, mut repo) = setup_repo_with_file("original", "file.txt")?;
        let file_path = dir.path().join("file.txt");

        fs::write(&file_path, "modified")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;

        RestoreCommand::new(&mut repo, "file.txt".to_string()).run()?;

        assert_eq!(fs::read_to_string(&file_path)?, "modified");

        Ok(())
    }

    #[test]
    fn restore_staged_deletion() -> anyhow::Result<()> {
        let (dir, mut repo) = setup_repo_with_file("original", "file.txt")?;
        let file_path = dir.path().join("file.txt");

        fs::remove_file(&file_path)?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;

        let mut index = Index::new(repo.flux_dir.join("index"));
        index.load()?;
        assert!(index.entries.get(&("file.txt".to_string(), 0)).is_none());

        RestoreCommand::new(&mut repo, "file.txt".to_string()).run()?;

        let mut index = Index::new(repo.flux_dir.join("index"));
        index.load()?;
        assert!(index.entries.get(&("file.txt".to_string(), 0)).is_some());
        assert!(!file_path.exists());

        Ok(())
    }

    #[test]
    fn restore_untracked_file_errors() -> anyhow::Result<()> {
        let (dir, mut repo) = setup_repo_with_file("original", "file.txt")?;
        fs::write(dir.path().join("untracked.txt"), "data")?;

        let result = RestoreCommand::new(&mut repo, "untracked.txt".to_string()).run();
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn restore_nested_file() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;
        repo.set("user_name".to_string(), "test".to_string())?;
        repo.set("user_email".to_string(), "test@test.com".to_string())?;

        let nested = dir.path().join("src/main.rs");
        fs::create_dir_all(nested.parent().unwrap())?;
        fs::write(&nested, "original")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;
        CommitCommand::new(&mut repo, "commit".to_string())?.run()?;

        fs::write(&nested, "modified")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;

        RestoreCommand::new(&mut repo, "src/main.rs".to_string()).run()?;

        assert_eq!(fs::read_to_string(&nested)?, "modified");

        let mut index = Index::new(repo.flux_dir.join("index"));
        index.load()?;
        let entry = index.entries.get(&("src/main.rs".to_string(), 0)).unwrap();
        let db = crate::database::database::Database::open(repo.flux_dir.clone());
        let obj = db.read_object(&hex::encode(entry.id))?;
        let blob = obj
            .as_any()
            .downcast_ref::<crate::database::blob::Blob>()
            .unwrap();
        assert_eq!(blob.as_string(), "original");

        Ok(())
    }

    #[test]
    fn restore_no_commits_removes_from_index() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        let file_path = dir.path().join("file.txt");
        fs::write(&file_path, "data")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;

        let mut index = Index::new(repo.flux_dir.join("index"));
        index.load()?;
        assert!(index.entries.get(&("file.txt".to_string(), 0)).is_some());

        RestoreCommand::new(&mut repo, "file.txt".to_string()).run()?;

        let mut index = Index::new(repo.flux_dir.join("index"));
        index.load()?;
        assert!(index.entries.get(&("file.txt".to_string(), 0)).is_none());

        Ok(())
    }
}
