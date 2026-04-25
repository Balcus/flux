use crate::database::blob::Blob;
use crate::database::database::Database;
use crate::database::object::Object;
use crate::dircache::index::Index;
use crate::internals::repository::Repository;
use crate::internals::work_tree::WorkTree;
use crate::status::change_type::ChangeType;
use anyhow::Result;
use anyhow::anyhow;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct Status {
    pub head_tree: HashMap<String, String>,
    pub index_changes: HashMap<String, ChangeType>,
    pub workspace_changes: HashMap<String, ChangeType>,
    pub untracked: Vec<String>,
}

impl Status {
    pub fn new(repo: &Repository) -> Result<Self> {
        let mut index = Index::new(repo.flux_dir.join("index"));
        index.load()?;

        let flat: HashMap<String, String> = index
            .entries
            .iter()
            .filter(|((_, stage), _)| *stage == 0)
            .map(|((path, _), entry)| (path.clone(), hex::encode(entry.id)))
            .collect();

        let head_tree = Self::get_head_snapshot(repo)?;
        let index_changes = Self::compare_head_to_index(&head_tree, &flat);
        let workspace_changes = Self::compare_index_to_workspace(&repo.work_tree, &flat)?;
        let untracked = Self::find_untracked_files(repo, &flat)?;

        Ok(Self {
            head_tree,
            index_changes,
            workspace_changes,
            untracked,
        })
    }

    fn get_head_snapshot(repo: &Repository) -> Result<HashMap<String, String>> {
        let db = Database::open(repo.flux_dir.clone());
        match repo.refs.head_commit() {
            Ok(hash) if !hash.is_empty() => Ok(db.commit_to_map(hash)?),
            _ => Ok(HashMap::new()),
        }
    }

    fn compare_head_to_index(
        head_snapshot: &HashMap<String, String>,
        index: &HashMap<String, String>,
    ) -> HashMap<String, ChangeType> {
        let mut changes = HashMap::new();

        for (path, index_hash) in index {
            match head_snapshot.get(path) {
                Some(head_hash) if head_hash != index_hash => {
                    changes.insert(path.clone(), ChangeType::Modified);
                }
                None => {
                    changes.insert(path.clone(), ChangeType::Added);
                }
                _ => {}
            }
        }

        for path in head_snapshot.keys() {
            if !index.contains_key(path) {
                changes.insert(path.clone(), ChangeType::Deleted);
            }
        }

        changes
    }

    fn compare_index_to_workspace(
        work_tree: &WorkTree,
        index: &HashMap<String, String>,
    ) -> Result<HashMap<String, ChangeType>> {
        let mut changes = HashMap::new();

        for (rel_path, index_hash) in index {
            let full_path = work_tree.path().join(rel_path);

            if !full_path.exists() {
                changes.insert(rel_path.clone(), ChangeType::Deleted);
            } else if full_path.is_file() {
                let data = work_tree.read_file(&full_path)?;
                let current_blob = Blob::from_bytes(data);
                if &current_blob.id() != index_hash {
                    changes.insert(rel_path.clone(), ChangeType::Modified);
                }
            }
        }

        Ok(changes)
    }

    fn find_untracked_files(
        repo: &Repository,
        index: &HashMap<String, String>,
    ) -> Result<Vec<String>> {
        let mut untracked = Vec::new();
        Self::scan_directory(
            repo.work_tree.path(),
            repo.work_tree.path(),
            index,
            &mut untracked,
        )?;
        untracked.sort();
        Ok(untracked)
    }

    fn scan_directory(
        root: &Path,
        current: &Path,
        index: &HashMap<String, String>,
        untracked: &mut Vec<String>,
    ) -> Result<()> {
        if current.ends_with(".flux") {
            return Ok(());
        }

        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();

            if path.ends_with(".flux") {
                continue;
            }

            let rel_path = path.strip_prefix(root)?;
            let rel_str = rel_path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid UTF-8 in path"))?;

            if path.is_file() {
                if !index.contains_key(rel_str) {
                    untracked.push(rel_str.to_string());
                }
            } else if path.is_dir() {
                Self::scan_directory(root, &path, index, untracked)?;
            }
        }

        Ok(())
    }

    pub fn is_clean(&self) -> bool {
        self.index_changes.is_empty()
            && self.workspace_changes.is_empty()
            && self.untracked.is_empty()
    }

    pub fn has_staged_changes(&self) -> bool {
        !self.index_changes.is_empty()
    }

    pub fn has_unstaged_changes(&self) -> bool {
        !self.workspace_changes.is_empty()
    }

    pub fn has_untracked_files(&self) -> bool {
        !self.untracked.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::{add::AddCommand, command::Command, commit::CommitCommand, init::InitCommand};

    use super::*;
    use std::{fs, path::PathBuf};
    use tempfile::tempdir;

    #[test]
    fn untracked_files() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();

        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let repo = Repository::open(Some(repo_path))?;
        fs::write(dir.path().join("test1.txt"), "hello world")?;
        let status: Status = Status::new(&repo)?;

        assert_eq!(status.untracked, vec!["test1.txt"]);
        assert!(status.index_changes.is_empty());
        assert!(status.workspace_changes.is_empty());

        fs::write(dir.path().join("test2.txt"), "hello world")?;
        fs::write(dir.path().join("test3.txt"), "hello world")?;
        fs::write(dir.path().join("test4.txt"), "hello world")?;
        fs::write(dir.path().join("test5.txt"), "hello world")?;
        fs::write(dir.path().join("test6.txt"), "hello world")?;
        fs::write(dir.path().join("test7.txt"), "hello world")?;

        let status = Status::new(&repo)?;
        assert_eq!(
            status.untracked,
            vec![
                "test1.txt",
                "test2.txt",
                "test3.txt",
                "test4.txt",
                "test5.txt",
                "test6.txt",
                "test7.txt",
            ]
        );

        fs::remove_file(dir.path().join("test1.txt"))?;
        fs::remove_file(dir.path().join("test3.txt"))?;
        fs::remove_file(dir.path().join("test5.txt"))?;
        fs::remove_file(dir.path().join("test7.txt"))?;

        let status = Status::new(&repo)?;
        assert_eq!(
            status.untracked,
            vec!["test2.txt", "test4.txt", "test6.txt",]
        );

        Ok(())
    }

    #[test]
    fn workspace_add_then_modify_file() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        let file_name = "test.txt";
        let file_path = dir.path().join(file_name);
        fs::write(&file_path, "hello world")?;

        AddCommand {
            repo: &mut repo,
            path: PathBuf::from(file_name),
        }
        .run()?;

        let status = Status::new(&repo)?;
        assert!(status.untracked.is_empty());
        assert!(status.workspace_changes.is_empty());
        assert_eq!(
            status.index_changes.get(file_name),
            Some(&ChangeType::Added)
        );

        fs::write(&file_path, "overwriting contents of the old test file.")?;
        let status = Status::new(&repo)?;
        assert_eq!(
            status.workspace_changes.get(file_name),
            Some(&ChangeType::Modified)
        );

        AddCommand {
            repo: &mut repo,
            path: PathBuf::from(file_name),
        }
        .run()?;
        let status = Status::new(&repo)?;
        assert!(status.workspace_changes.is_empty());

        Ok(())
    }

    #[test]
    fn index_add_delete_after_commit() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        let file_name = "test.txt";
        let file_path = dir.path().join(file_name);
        fs::write(&file_path, "hello world")?;

        AddCommand {
            repo: &mut repo,
            path: PathBuf::from(file_name),
        }
        .run()?;
        repo.set("user_name".to_string(), "test_user".to_string())?;
        repo.set("user_email".to_string(), "test_user@email.com".to_string())?;
        CommitCommand::new(&mut repo, "initial commit".to_string())?.run()?;

        fs::remove_file(&file_path)?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;
        let status = Status::new(&repo)?;

        assert!(status.untracked.is_empty());
        assert_eq!(
            status.index_changes.get(file_name),
            Some(&ChangeType::Deleted)
        );
        assert!(status.workspace_changes.is_empty());

        Ok(())
    }

    #[test]
    fn workspace_delete_added_file() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        let file_name = "test.txt";
        let file_path = dir.path().join(file_name);

        fs::write(&file_path, "hello world")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from(file_name),
        }
        .run()?;

        fs::remove_file(&file_path)?;

        let status = Status::new(&repo)?;
        assert_eq!(
            status.workspace_changes.get(file_name),
            Some(&ChangeType::Deleted)
        );
        assert_eq!(
            status.index_changes.get(file_name),
            Some(&ChangeType::Added)
        );
        assert!(status.untracked.is_empty());

        Ok(())
    }

    #[test]
    fn workspace_multiple_modifications() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        let files = ["a.txt", "b.txt", "c.txt"];
        for file in &files {
            fs::write(dir.path().join(file), "original content")?;
            AddCommand {
                repo: &mut repo,
                path: PathBuf::from(file),
            }
            .run()?;
        }

        fs::write(dir.path().join("a.txt"), "changed content")?;
        fs::write(dir.path().join("c.txt"), "other changed changed")?;

        let status = Status::new(&repo)?;
        assert_eq!(
            status.workspace_changes.get("a.txt"),
            Some(&ChangeType::Modified)
        );
        assert_eq!(status.workspace_changes.get("b.txt"), None);
        assert_eq!(
            status.workspace_changes.get("c.txt"),
            Some(&ChangeType::Modified)
        );
        assert_eq!(status.workspace_changes.len(), 2);

        Ok(())
    }

    #[test]
    fn index_added_multiple_files() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        repo.set("user_name".to_string(), "test_user".to_string())?;
        repo.set("user_email".to_string(), "test_user@email.com".to_string())?;

        fs::write(dir.path().join("existing.txt"), "existing")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("existing.txt"),
        }
        .run()?;
        CommitCommand::new(&mut repo, "initial commit".to_string())?.run()?;

        fs::write(dir.path().join("new1.txt"), "new file 1")?;
        fs::write(dir.path().join("new2.txt"), "new file 2")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("new1.txt"),
        }
        .run()?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("new2.txt"),
        }
        .run()?;

        let status = Status::new(&repo)?;
        assert_eq!(
            status.index_changes.get("new1.txt"),
            Some(&ChangeType::Added)
        );
        assert_eq!(
            status.index_changes.get("new2.txt"),
            Some(&ChangeType::Added)
        );
        assert_eq!(status.index_changes.get("existing.txt"), None);

        Ok(())
    }

    #[test]
    fn index_modified_file() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        repo.set("user_name".to_string(), "test_user".to_string())?;
        repo.set("user_email".to_string(), "test_user@email.com".to_string())?;

        let file_name = "test.txt";
        let file_path = dir.path().join(file_name);
        fs::write(&file_path, "original content")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from(file_name),
        }
        .run()?;
        CommitCommand::new(&mut repo, "initial commit".to_string())?.run()?;

        fs::write(&file_path, "modified content")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from(file_name),
        }
        .run()?;

        let status = Status::new(&repo)?;
        assert_eq!(
            status.index_changes.get(file_name),
            Some(&ChangeType::Modified)
        );
        assert!(status.workspace_changes.is_empty());
        assert!(status.untracked.is_empty());

        Ok(())
    }

    #[test]
    fn is_clean_after_commit() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        repo.set("user_name".to_string(), "test_user".to_string())?;
        repo.set("user_email".to_string(), "test_user@email.com".to_string())?;

        fs::write(dir.path().join("test.txt"), "hello")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("test.txt"),
        }
        .run()?;
        CommitCommand::new(&mut repo, "initial commit".to_string())?.run()?;

        let status = Status::new(&repo)?;
        assert!(status.is_clean());
        assert!(!status.has_staged_changes());
        assert!(!status.has_unstaged_changes());
        assert!(!status.has_untracked_files());

        Ok(())
    }

    #[test]
    fn staged_and_unstaged_changes_to_same_file() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        repo.set("user_name".to_string(), "test_user".to_string())?;
        repo.set("user_email".to_string(), "test_user@email.com".to_string())?;

        let file_name = "test.txt";
        let file_path = dir.path().join(file_name);
        fs::write(&file_path, "original")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from(file_name),
        }
        .run()?;
        CommitCommand::new(&mut repo, "initial commit".to_string())?.run()?;

        fs::write(&file_path, "staged change")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from(file_name),
        }
        .run()?;

        fs::write(&file_path, "unstaged change on top")?;

        let status = Status::new(&repo)?;
        assert_eq!(
            status.index_changes.get(file_name),
            Some(&ChangeType::Modified)
        );
        assert_eq!(
            status.workspace_changes.get(file_name),
            Some(&ChangeType::Modified)
        );

        Ok(())
    }

    #[test]
    fn empty_repo_is_clean() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let repo = Repository::open(Some(repo_path))?;

        let status = Status::new(&repo)?;
        assert!(status.is_clean());
        assert!(status.index_changes.is_empty());
        assert!(status.workspace_changes.is_empty());
        assert!(status.untracked.is_empty());

        Ok(())
    }

    #[test]
    fn reverting_changes() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        repo.set("user_name".to_string(), "test_user".to_string())?;
        repo.set("user_email".to_string(), "test_user@email.com".to_string())?;

        let file_name = "test.txt";
        let file_path = dir.path().join(file_name);
        fs::write(&file_path, "original content")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from(file_name),
        }
        .run()?;
        CommitCommand::new(&mut repo, "initial commit".to_string())?.run()?;

        fs::write(&file_path, "temporary change")?;
        fs::write(&file_path, "original content")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from(file_name),
        }
        .run()?;

        let status = Status::new(&repo)?;
        assert_eq!(status.index_changes.get(file_name), None);
        assert!(status.workspace_changes.is_empty());
        assert!(status.is_clean());

        Ok(())
    }

    #[test]
    fn mixed_operations() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        repo.set("user_name".to_string(), "test_user".to_string())?;
        repo.set("user_email".to_string(), "test_user@email.com".to_string())?;

        let modify_path = dir.path().join("modify_me.txt");
        let delete_path = dir.path().join("delete_me.txt");
        fs::write(&modify_path, "original")?;
        fs::write(&delete_path, "to be deleted")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("modify_me.txt"),
        }
        .run()?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("delete_me.txt"),
        }
        .run()?;
        CommitCommand::new(&mut repo, "initial commit".to_string())?.run()?;

        fs::write(&modify_path, "changed")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("modify_me.txt"),
        }
        .run()?;
        fs::remove_file(&delete_path)?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;
        fs::write(dir.path().join("new_untracked.txt"), "untracked")?;
        fs::write(dir.path().join("new_staged.txt"), "staged")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("new_staged.txt"),
        }
        .run()?;

        let status = Status::new(&repo)?;
        assert_eq!(
            status.index_changes.get("modify_me.txt"),
            Some(&ChangeType::Modified)
        );
        assert_eq!(
            status.index_changes.get("delete_me.txt"),
            Some(&ChangeType::Deleted)
        );
        assert_eq!(
            status.index_changes.get("new_staged.txt"),
            Some(&ChangeType::Added)
        );
        assert_eq!(status.index_changes.len(), 3);
        assert!(status.workspace_changes.is_empty());
        assert_eq!(status.untracked, vec!["new_untracked.txt"]);

        Ok(())
    }

    #[test]
    fn nested_directories() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().to_string_lossy().to_string();
        InitCommand::new(Some(repo_path.clone()), true).run()?;
        let mut repo = Repository::open(Some(repo_path))?;

        repo.set("user_name".to_string(), "test_user".to_string())?;
        repo.set("user_email".to_string(), "test_user@email.com".to_string())?;

        fs::create_dir_all(dir.path().join("src/utils"))?;
        fs::write(dir.path().join("src/main.rs"), "fn main() {}")?;
        fs::write(dir.path().join("src/utils/helper.rs"), "fn help() {}")?;
        fs::write(dir.path().join("readme.md"), "# project")?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;
        CommitCommand::new(&mut repo, "initial commit".to_string())?.run()?;

        let status = Status::new(&repo)?;
        assert!(status.is_clean());

        fs::write(
            dir.path().join("src/utils/helper.rs"),
            "fn help() { todo!() }",
        )?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("src/utils/helper.rs"),
        }
        .run()?;

        fs::remove_file(dir.path().join("src/main.rs"))?;
        AddCommand {
            repo: &mut repo,
            path: PathBuf::from("."),
        }
        .run()?;

        fs::write(dir.path().join("src/new.rs"), "fn new() {}")?;
        fs::create_dir_all(dir.path().join("src/utils/extra"))?;
        fs::write(dir.path().join("src/utils/extra/deep.rs"), "fn deep() {}")?;

        let status = Status::new(&repo)?;
        assert_eq!(
            status.index_changes.get("src/utils/helper.rs"),
            Some(&ChangeType::Modified)
        );
        assert_eq!(
            status.index_changes.get("src/main.rs"),
            Some(&ChangeType::Deleted)
        );
        assert_eq!(status.index_changes.get("readme.md"), None);
        assert!(status.untracked.contains(&"src/new.rs".to_string()));
        assert!(
            status
                .untracked
                .contains(&"src/utils/extra/deep.rs".to_string())
        );
        assert!(status.workspace_changes.is_empty());

        Ok(())
    }
}
