use crate::{
    commands::command::Command,
    database::{commit::Commit, database::Database, object::Object},
    dircache::index::Index,
    internals::repository::Repository,
    status::status_impl::Status,
};
use anyhow::Context;
use chrono::{DateTime, Utc};

pub struct Author {
    pub user_name: String,
    pub user_email: String,
}

pub struct CommitCommand<'a> {
    pub repo: &'a mut Repository,
    pub message: String,
    pub author: Author,
    pub time: DateTime<Utc>,
    pub database: Database,
}

impl<'a> CommitCommand<'a> {
    pub fn new(repo: &'a mut Repository, message: String) -> anyhow::Result<Self> {
        let credentials = repo
            .config
            .get_credentials()
            .context("Failed to get credentials from config")?;

        let author = Author {
            user_name: credentials.user_name,
            user_email: credentials.user_email,
        };

        Ok(Self {
            database: Database::open(repo.flux_dir.clone()),
            repo,
            message,
            author,
            time: Utc::now(),
        })
    }
}

impl<'a> Command for CommitCommand<'a> {
    fn run(&mut self) -> anyhow::Result<()> {
        let status = Status::new(self.repo)?;
        let mut index = Index::new(self.repo.flux_dir.join("index"));
        index.load()?;

        if status.is_clean() {
            anyhow::bail!("Nothing to commit, index is empty.");
        }

        let tree_hash = self
            .repo
            .work_tree
            .build_tree_from_index(&index, &self.database)
            .context("Failed to build tree from index")?;

        let parent_hash = self.repo.refs.head_commit().ok().filter(|s| !s.is_empty());

        let commit = Commit::new(
            tree_hash,
            self.author.user_name.clone(),
            self.author.user_email.clone(),
            parent_hash,
            self.message.clone(),
        );

        let commit_id = commit.id();
        self.database.store(Box::new(commit.clone()))?;
        self.repo.refs.update_head(&commit_id)?;

        println!("{commit_id}");

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        commands::{add::AddCommand, command::Command, commit::CommitCommand, init::InitCommand},
        database::{blob::Blob, commit::Commit, database::Database, object::Object, tree::Tree},
        internals::repository::Repository,
        utils::modes::{MODE_DIR, MODE_FILE},
    };
    use anyhow::Context;
    use std::{fs, path::PathBuf};
    use tempfile::tempdir;

    #[test]
    fn commit_with_parent_and_mode() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo_path_str = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path_str.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path_str))?;

        repo.set("user_name".to_string(), "test_user".to_string())?;
        repo.set("user_email".to_string(), "test@email.com".to_string())?;

        let main_content = "fn main() {println!(\"Hello World\");}";
        let readme_content = "Readme file";
        let src_path = dir.path().join("src");
        fs::create_dir_all(&src_path)?;
        fs::write(src_path.join("main.rs"), main_content)?;
        fs::write(src_path.join("lib.rs"), "")?;
        fs::write(dir.path().join("README.md"), readme_content)?;

        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;
        CommitCommand::new(&mut repo, "Initial commit".to_string())?.run()?;

        let first_commit_id = repo.refs.head_commit()?;
        let updated_readme = "Updated Readme file";
        fs::write(dir.path().join("README.md"), updated_readme)?;

        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;
        CommitCommand::new(&mut repo, "Second commit".to_string())?.run()?;

        let second_commit_id = repo.refs.head_commit()?;
        let db = Database::open(repo.flux_dir.clone());
        let obj = db.read_object(&second_commit_id)?;
        let second_commit = obj.as_any().downcast_ref::<Commit>().expect("Not a commit");
        assert_eq!(second_commit.parent_hash(), Some(first_commit_id.as_str()));

        let file_map = db.commit_to_map(second_commit_id)?;
        let expected_files = vec![
            ("src/main.rs", main_content),
            ("README.md", updated_readme),
            ("src/lib.rs", ""),
        ];

        for (path, expected_content) in expected_files {
            let blob_id = file_map
                .get(path)
                .context(format!("Missing path: {}", path))?;
            let blob_obj = db.read_object(blob_id)?;
            let blob = blob_obj
                .as_any()
                .downcast_ref::<Blob>()
                .expect("Not a blob");
            assert_eq!(String::from_utf8_lossy(&blob.content()), expected_content);
        }

        let tree_obj = db.read_object(&second_commit.tree_hash)?;
        let tree = tree_obj
            .as_any()
            .downcast_ref::<Tree>()
            .expect("Not a tree");
        for entry in tree.entries() {
            if entry.is_dir() {
                assert_eq!(entry.mode, MODE_DIR);
            } else {
                assert_eq!(entry.mode, MODE_FILE);
            }
        }

        Ok(())
    }

    #[test]
    fn commit_without_credentials() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let repo_path_str = dir.path().to_string_lossy().to_string();

        InitCommand::new(Some(repo_path_str.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path_str))?;
        fs::write(
            dir.path().join("main.rs"),
            "fn main() {println!(\"Hello World\");}",
        )?;
        
        AddCommand { repo: &mut repo, path: PathBuf::from(".") }.run()?;

        let res = repo.commit("Initial commit".to_string());
        let err = res.unwrap_err();
        assert!(err.to_string().contains("credentials"));
        Ok(())
    }
}
