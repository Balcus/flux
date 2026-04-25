use flux_core::commands::add::AddCommand;
use flux_core::commands::command::Command;
use flux_core::commands::commit::CommitCommand;
use flux_core::commands::hash_object::HashObject;
use flux_core::commands::init::InitCommand;
use flux_core::database::database::Database;
use flux_core::dircache::index::Index;
use flux_core::internals::repository::Repository;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;

mod common;

#[test]
#[serial]
fn project_creation() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    assert!(project_path.join("README.md").exists());
    assert!(project_path.join("src/main.rs").exists());
    assert!(project_path.join("src/lib.rs").exists());

    let readme = fs::read_to_string("README.md").unwrap();
    let main_rs = fs::read_to_string("src/main.rs").unwrap();
    let lib_rs = fs::read_to_string("src/lib.rs").unwrap();

    assert_eq!(readme, "Read this file before running the project");
    assert_eq!(main_rs, r#"pub fn main() { println!("{}", add(1, 2)) }"#);
    assert_eq!(lib_rs, "pub fn add(a: i32, b: i32) -> i64 { a + b }");
}

#[test]
#[serial]
fn open_without_repo() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    let err = Repository::open(None).unwrap_err();
    assert!(err.to_string().contains("not initialized"), "got: {err}");
    println!("{err}");
}

#[test]
#[serial]
fn set() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    InitCommand::new(None, true).run().unwrap();
    let mut repo = Repository::open(None).unwrap();
    repo.set("user_name".to_string(), "user".to_string())
        .unwrap();
    repo.set("user_email".to_string(), "user@gmail.com".to_string())
        .unwrap();

    assert!(project_path.join(".flux/config").exists());

    let config = fs::read_to_string(".flux/config").unwrap();
    assert!(config.contains("user_name = \"user\""));
    assert!(config.contains("user_email = \"user@gmail.com\""));
}

#[test]
#[serial]
fn commit() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    InitCommand::new(None, true).run().unwrap();
    let mut repo = Repository::open(None).unwrap();

    repo.set("user_name".to_string(), "Test User".to_string())
        .expect("Failed to set user name");
    repo.set("user_email".to_string(), "test@example.com".to_string())
        .expect("Failed to set user email");

    let readme_blob_hash = HashObject::new("./README.md", false)
        .hash(None)
        .expect("Failed to hash file README.md");
    let readme_object_path = project_path
        .join(".flux/objects")
        .join(&readme_blob_hash[..2])
        .join(&readme_blob_hash[2..]);
    assert!(!readme_object_path.exists());

    AddCommand {
        repo: &mut repo,
        path: PathBuf::from("./README.md"),
    }
    .run()
    .expect("Failed to add README to index");

    let mut index = Index::new(repo.flux_dir.join("index"));
    index.load().unwrap();
    let entry = index
        .entries
        .get(&("README.md".to_string(), 0))
        .expect("Failed to find README inside index");
    assert_eq!(hex::encode(entry.id), readme_blob_hash);
    assert!(readme_object_path.exists());

    AddCommand {
        repo: &mut repo,
        path: PathBuf::from("./src"),
    }
    .run()
    .expect("Failed to add src to index");

    let main_hash = HashObject::new("./src/main.rs", false)
        .hash(None)
        .expect("Failed to hash file ./src/main.rs");
    let lib_hash = HashObject::new("./src/lib.rs", false)
        .hash(None)
        .expect("Failed to hash file ./src/lib.rs");

    index.load().unwrap();
    assert_eq!(
        hex::encode(
            index
                .entries
                .get(&("src/main.rs".to_string(), 0))
                .expect("main.rs not in index")
                .id
        ),
        main_hash
    );
    assert_eq!(
        hex::encode(
            index
                .entries
                .get(&("src/lib.rs".to_string(), 0))
                .expect("lib.rs not in index")
                .id
        ),
        lib_hash
    );

    let main_object_path = project_path
        .join(".flux/objects")
        .join(&main_hash[..2])
        .join(&main_hash[2..]);
    assert!(main_object_path.exists());

    let commit_hash = CommitCommand::new(&mut repo, "Initial commit".to_string())
        .unwrap()
        .run()
        .expect("Failed to create initial commit");
    let commit_object_path = project_path
        .join(".flux/objects")
        .join(&commit_hash[..2])
        .join(&commit_hash[2..]);
    assert!(commit_object_path.exists());

    let head_content = fs::read_to_string(".flux/HEAD").unwrap();
    assert_eq!(head_content.trim(), "ref: refs/heads/main");

    let main_ref = fs::read_to_string(".flux/refs/heads/main").unwrap();
    assert_eq!(main_ref.trim(), commit_hash);

    let db = Database::open(repo.flux_dir.clone());
    let commit_content = String::from_utf8(db.read_object(&commit_hash).unwrap().content())
        .expect("Failed to read commit content");

    assert!(commit_content.starts_with("tree "));
    assert!(commit_content.contains("author Test User <test@example.com>"));
    assert!(commit_content.contains("committer Test User <test@example.com>"));
    assert!(commit_content.contains("Initial commit"));
    assert!(!commit_content.contains("parent "));

    let tree_line = commit_content.lines().next().unwrap();
    let tree_hash = tree_line.strip_prefix("tree ").unwrap().trim();
    let tree_object_path = project_path
        .join(".flux/objects")
        .join(&tree_hash[..2])
        .join(&tree_hash[2..]);
    assert!(tree_object_path.exists());

    fs::write("README.md", "Updated content for second commit").unwrap();
    AddCommand {
        repo: &mut repo,
        path: PathBuf::from("README.md"),
    }
    .run()
    .unwrap();

    let second_commit_hash = CommitCommand::new(&mut repo, "Second commit".to_string())
        .unwrap()
        .run()
        .expect("Failed to create second commit");
    assert_ne!(commit_hash, second_commit_hash);

    let main_ref = fs::read_to_string(".flux/refs/heads/main").expect("Failed to read HEAD");
    assert_eq!(main_ref.trim(), second_commit_hash);

    let second_commit_content =
        String::from_utf8(db.read_object(&second_commit_hash).unwrap().content())
            .expect("Failed to read second commit content to string");

    assert!(second_commit_content.contains(&format!("parent {}", commit_hash)));
    assert!(second_commit_content.contains("Second commit"));
}

#[test]
#[serial]
fn commit_with_empty_index() {
    let (_temp, _project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&_project_path).unwrap();

    InitCommand::new(None, true).run().unwrap();
    let mut repo = Repository::open(None).unwrap();

    let res = CommitCommand::new(&mut repo, "commit".to_string()).and_then(|mut cmd| cmd.run());
    assert!(res.is_err());
}

#[test]
#[serial]
fn commit_without_credentials() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    InitCommand::new(None, true).run().unwrap();
    let mut repo = Repository::open(None).unwrap();

    AddCommand {
        repo: &mut repo,
        path: PathBuf::from("README.md"),
    }
    .run()
    .unwrap();

    let res = CommitCommand::new(&mut repo, "commit".to_string()).and_then(|mut cmd| cmd.run());
    assert!(res.is_err());
}

#[test]
#[serial]
fn branching() -> anyhow::Result<()> {
    let (_temp, _project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&_project_path).unwrap();

    InitCommand::new(None, true).run()?;
    let mut repo = Repository::open(None)?;

    repo.set(String::from("user_name"), String::from("test"))?;
    repo.set(String::from("user_email"), String::from("test@gmail.com"))?;

    AddCommand {
        repo: &mut repo,
        path: PathBuf::from("."),
    }
    .run()?;

    let first_commit_hash =
        CommitCommand::new(&mut repo, "First commit on branch main".to_string())?.run()?;

    assert_eq!(repo.refs.head_ref()?, "refs/heads/main");
    assert_eq!(repo.refs.current_branch()?, "main");

    let head_content = fs::read_to_string(_project_path.join(".flux").join(repo.refs.head_ref()?))?;
    assert_eq!(head_content, first_commit_hash);

    repo.new_branch("feature")?;
    assert!(fs::exists(repo.flux_dir.join("refs/heads/feature"))?);
    let feature_content = fs::read_to_string(repo.flux_dir.join("refs/heads/feature"))?;
    assert_eq!(feature_content, head_content);

    assert!(fs::exists(repo.work_tree.path().join("README.md"))?);
    assert!(fs::exists(repo.work_tree.path().join("src/main.rs"))?);
    assert!(fs::exists(repo.work_tree.path().join("src/lib.rs"))?);

    repo.switch_branch("main", false)?;
    assert!(fs::exists(repo.work_tree.path().join("README.md"))?);
    assert!(fs::exists(repo.work_tree.path().join("src/main.rs"))?);
    assert!(fs::exists(repo.work_tree.path().join("src/lib.rs"))?);

    fs::write("README.md", "Added something new to README")?;
    AddCommand {
        repo: &mut repo,
        path: PathBuf::from("."),
    }
    .run()?;

    let second_commit_hash =
        CommitCommand::new(&mut repo, "Second commit on main branch".to_string())?.run()?;

    let main_head = fs::read_to_string(repo.flux_dir.join("refs/heads/main"))?;
    assert_eq!(second_commit_hash, main_head);
    assert_eq!(second_commit_hash, repo.refs.head_commit()?);

    repo.switch_branch("feature", false)?;
    assert_eq!(repo.refs.current_branch()?, "feature");
    assert_eq!(first_commit_hash, repo.refs.head_commit()?);
    assert!(
        fs::read_to_string("./README.md")?.contains("Read this file before running the project")
    );
    assert!(!fs::read_to_string("./README.md")?.contains("Added something new to README"));

    Ok(())
}

#[test]
#[serial]
fn branching_errors() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    InitCommand::new(None, true).run().unwrap();
    let mut repo = Repository::open(None).unwrap();

    let err = repo.delete_branch("main").unwrap_err();
    assert!(
        err.to_string().contains("Cannot delete the current branch"),
        "got: {err}"
    );
    println!("{err}");

    assert!(repo.switch_branch("does-not-exist", false).is_err());

    let err = repo.new_branch("main").unwrap_err();
    assert!(err.to_string().contains("already exists"), "got: {err}");
    println!("{err}");

    fs::remove_dir_all(&repo.refs.refs_path).unwrap();
    let err = Repository::open(None).unwrap_err();
    assert!(
        err.to_string().contains("Missing required path"),
        "got: {err}"
    );
    println!("{err}");

    fs::write(&repo.refs.head_path, "invalidate head").unwrap();
    let err = repo.show_branches().unwrap_err();
    assert!(
        err.to_string().contains("Invalid head format"),
        "got: {err}"
    );
    println!("{err}");
}
