use crate::{
    commands::command::Command,
    database::{blob::Blob, database::Database, walker::Walker},
    dircache::index::Index,
    internals::repository::Repository,
};
use std::path::PathBuf;

pub struct ResetCommand<'a> {
    pub repo: &'a mut Repository,
    pub path: PathBuf,
    pub hard: bool,
}

impl<'a> ResetCommand<'a> {
    pub fn new(repo: &'a mut Repository, path_s: String, hard: bool) -> Self {
        ResetCommand {
            repo,
            path: PathBuf::from(path_s),
            hard,
        }
    }

    fn resolve_path(&self) -> anyhow::Result<PathBuf> {
        if self.path.is_absolute() {
            let abs = self.path.canonicalize()?;
            Ok(abs.strip_prefix(self.repo.work_tree.path())?.to_path_buf())
        } else {
            Ok(self.path.clone())
        }
    }

    pub fn soft_reset(&mut self, index: &Index, rel_path: &PathBuf) -> anyhow::Result<()> {
        let path_str = rel_path.to_string_lossy().to_string();

        let entry = index
            .entries
            .get(&(path_str.clone(), 0))
            .ok_or_else(|| anyhow::anyhow!("File not found in index"))?;

        let db = Database::open(self.repo.flux_dir.clone());
        let obj = db.read_object(&hex::encode(entry.id))?;

        let blob = obj
            .as_any()
            .downcast_ref::<Blob>()
            .ok_or_else(|| anyhow::anyhow!("Object is not a blob"))?;

        self.repo.work_tree.write_file(
            rel_path,
            blob.as_string().as_bytes(),
            Some(entry.mode),
            true,
        )?;

        Ok(())
    }

    pub fn hard_reset(&mut self, index: &mut Index, rel_path: &PathBuf) -> anyhow::Result<()> {
        let head_commit = self.repo.refs.head_commit()?;
        if head_commit.is_empty() {
            anyhow::bail!("No commits yet — nothing to hard reset to");
        }

        let path_str = rel_path.to_string_lossy().to_string();

        let db = Database::open(self.repo.flux_dir.clone());
        let walker = Walker::new(&db);
        let blob_hash = walker
            .file_hash_from_commit(&path_str, &head_commit)?
            .ok_or_else(|| anyhow::anyhow!("'{}' not found in HEAD commit", path_str))?;

        let obj = db.read_object(&blob_hash)?;
        let blob = obj
            .as_any()
            .downcast_ref::<Blob>()
            .ok_or_else(|| anyhow::anyhow!("Object is not a blob"))?;

        let mode = index.entries.get(&(path_str.clone(), 0)).map(|e| e.mode);

        self.repo
            .work_tree
            .write_file(rel_path, blob.as_string().as_bytes(), mode, true)?;

        let abs_path = self.repo.work_tree.path().join(rel_path);
        let stat = std::fs::metadata(&abs_path)?;
        index.add(path_str, blob_hash, stat)?;
        index.write_updates()?;

        Ok(())
    }
}

impl<'a> Command for ResetCommand<'a> {
    fn run(&mut self) -> anyhow::Result<()> {
        let mut index = Index::new(self.repo.flux_dir.join("index"));
        index.load()?;

        let rel_path = self.resolve_path()?;

        if self.hard {
            self.hard_reset(&mut index, &rel_path)
        } else {
            self.soft_reset(&index, &rel_path)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        commands::{
            add::AddCommand, command::Command, commit::CommitCommand, init::InitCommand,
            reset::ResetCommand,
        },
        internals::repository::Repository,
    };
    use std::{fs, path::PathBuf};
    use tempfile::tempdir;

    #[test]
    fn soft_reset() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo_path_str = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path_str.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path_str))?;

        repo.set("user_name".to_string(), "test_user".to_string())?;
        repo.set("user_email".to_string(), "test@email.com".to_string())?;

        let initial_main_content = "fn main() {println!(\"Hello World\");}";
        let main_rs_path = dir.path().join("main.rs");
        fs::write(&main_rs_path, initial_main_content)?;

        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;

        CommitCommand::new(&mut repo, "Initial commit".to_string())?.run()?;

        let new_main_content = "fn main() {println!(\"{} + {} = {}\", 5, 10, 5 + 10)}";
        fs::write(&main_rs_path, new_main_content)?;

        assert_eq!(fs::read_to_string(&main_rs_path)?, new_main_content);
        ResetCommand::new(&mut repo, main_rs_path.to_string_lossy().to_string(), false).run()?;
        assert_eq!(fs::read_to_string(&main_rs_path)?, initial_main_content);

        Ok(())
    }

    #[test]
    fn soft_reset_untracked_file() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();

        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        let file_path = dir.path().join("untracked.txt");
        fs::write(&file_path, "data")?;

        let result =
            ResetCommand::new(&mut repo, file_path.to_string_lossy().to_string(), false).run();

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn soft_reset_nested_file() -> anyhow::Result<()> {
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

        ResetCommand::new(&mut repo, nested.to_string_lossy().to_string(), false).run()?;

        assert_eq!(fs::read_to_string(&nested)?, "original");

        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn soft_reset_preserves_permissions() -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();

        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        repo.set("user_name".to_string(), "test".to_string())?;
        repo.set("user_email".to_string(), "test@test.com".to_string())?;

        let file_path = dir.path().join("script.sh");
        fs::write(&file_path, "echo hi")?;

        let mut perms = fs::metadata(&file_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&file_path, perms)?;

        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;
        CommitCommand::new(&mut repo, "commit".to_string())?.run()?;

        fs::write(&file_path, "modified")?;

        ResetCommand::new(&mut repo, file_path.to_string_lossy().to_string(), false).run()?;

        let mode = fs::metadata(&file_path)?.permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);

        Ok(())
    }

    #[test]
    fn hard_reset() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo_path_str = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path_str.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path_str))?;

        repo.set("user_name".to_string(), "test_user".to_string())?;
        repo.set("user_email".to_string(), "test@email.com".to_string())?;

        let initial_content = "fn main() {println!(\"Hello World\");}";
        let file_path = dir.path().join("main.rs");
        fs::write(&file_path, initial_content)?;

        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;
        CommitCommand::new(&mut repo, "Initial commit".to_string())?.run()?;

        let staged_content = "fn main() {println!(\"staged\");}";
        fs::write(&file_path, staged_content)?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;

        assert_eq!(fs::read_to_string(&file_path)?, staged_content);
        ResetCommand::new(&mut repo, file_path.to_string_lossy().to_string(), true).run()?;
        assert_eq!(fs::read_to_string(&file_path)?, initial_content);

        Ok(())
    }
}
