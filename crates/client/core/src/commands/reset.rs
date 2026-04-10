use std::path::PathBuf;
use crate::{
    commands::command::Command,
    database::{blob::Blob, database::Database},
    dircache::index::Index,
    internals::repository::Repository,
};

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
            hard
        }
    }

    pub fn soft_reset(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn hard_reset(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl<'a> Command for ResetCommand<'a> {
    fn run(&mut self) -> anyhow::Result<()> {
        let mut index = Index::new(self.repo.flux_dir.join("index"));
        index.load()?;

        let rel_path = if self.path.is_absolute() {
            let abs = self.path.canonicalize()?;
            abs.strip_prefix(self.repo.work_tree.path())?.to_path_buf()
        } else {
            self.path.clone()
        };

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

        let data = blob.as_string();

        self.repo
            .work_tree
            .write_file(&rel_path, data.as_bytes(), Some(entry.mode), true)?;

        Ok(())
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

    // #[test]
    // fn reset_deleted_file() -> anyhow::Result<()> {
    //     let dir = tempdir()?;
    //     let repo_path = dir.path().to_string_lossy().to_string();

    //     InitCommand::new(Some(repo_path.clone()), true).run()?;
    //     let mut repo = Repository::open(Some(repo_path))?;

    //     repo.set("user_name".to_string(), "test".to_string())?;
    //     repo.set("user_email".to_string(), "test@test.com".to_string())?;

    //     let file_path = dir.path().join("file.txt");
    //     fs::write(&file_path, "original")?;

    //     AddCommand {
    //         repo: &mut repo,
    //         path: PathBuf::from("."),
    //     }
    //     .run()?;
    //     CommitCommand::new(&mut repo, "commit".to_string())?.run()?;

    //     fs::remove_file(&file_path)?;
    //     assert!(!file_path.exists());

    //     ResetCommand::new(&mut repo, file_path.to_string_lossy().to_string()).run()?;

    //     assert_eq!(fs::read_to_string(&file_path)?, "original");

    //     Ok(())
    // }

    #[test]
    fn soft_reset_untracked_file() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();

        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        let file_path = dir.path().join("untracked.txt");
        fs::write(&file_path, "data")?;

        let result = ResetCommand::new(&mut repo, file_path.to_string_lossy().to_string(), false).run();

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
}
