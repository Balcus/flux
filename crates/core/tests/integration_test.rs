use flux_core::{commands, internals::repository::Repository};
use serial_test::serial;
use std::fs;

mod common;

#[test]
#[serial]
fn project_creation_test() {
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
fn init_test() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    let repo = Repository::init(None, false).unwrap();

    assert!(project_path.join(".flux/config").exists());
    assert!(project_path.join(".flux/HEAD").exists());
    assert!(project_path.join(".flux/objects").exists());
    assert!(project_path.join(".flux/refs").exists());

    let head = fs::read_to_string(".flux/HEAD").unwrap();
    assert_eq!(head, "ref: refs/heads/main\n");
    assert_eq!(repo.refs.head_ref().unwrap(), "refs/heads/main");
}

#[test]
#[serial]
fn init_when_already_initialized() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    Repository::init(None, false).unwrap();
    let err = Repository::init(None, false).unwrap_err();

    match &err {
        flux_core::error::RepositoryError::AlreadyInitialized(path) => {
            assert!(path.ends_with(".flux"));
        }
        other => panic!("expected AlreadyInitialized error, got: {other:?}"),
    }
    println!("{err}");
}

#[test]
#[serial]
fn open_without_repo() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    let err = Repository::open(None).err().unwrap();
    match &err {
        flux_core::error::RepositoryError::NotRepository(path) => {
            let expected = project_path
                .canonicalize()
                .unwrap_or_else(|_| project_path.clone());

            assert_eq!(path, &expected);
            assert!(path.is_absolute());
        }
        other => panic!("expected NotRepository error, got: {other:?}"),
    }
    println!("{err}");
}

#[test]
#[serial]
fn set_test() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    Repository::init(None, false).unwrap();

    commands::set(None, "user_name".to_string(), "user".to_string()).unwrap();
    commands::set(None, "user_email".to_string(), "user@gmail.com".to_string()).unwrap();

    assert!(project_path.join(".flux/config").exists());

    let config = fs::read_to_string(".flux/config").unwrap();
    assert!(config.contains("user_name = \"user\""));
    assert!(config.contains("user_email = \"user@gmail.com\""));
}

#[test]
#[serial]
fn hash_object_test() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    Repository::init(None, false).unwrap();

    let my_hash = commands::hash_object(None, "README.md".to_string(), false).unwrap();
    let git_hash = common::git_hash_object("README.md").unwrap();
    assert_eq!(my_hash, git_hash);
    let object_path = project_path
        .join(".flux/objects")
        .join(&git_hash[..2])
        .join(&git_hash[2..]);
    assert!(!object_path.exists());
    let _ = commands::hash_object(None, "README.md".to_string(), true).unwrap();
    assert!(object_path.exists());

    assert!(project_path.join("src").exists());
    let my_hash = commands::hash_object(None, "src".to_string(), true).unwrap();
    assert_eq!(my_hash, "ac715a76cc52acc719def812525f6ae57b4770a9");
}

#[test]
#[serial]
fn commit_test() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    let mut repo = Repository::init(None, false).unwrap();
    repo.set("user_name".to_string(), "Test User".to_string())
        .expect("Failed to set user name");
    repo.set("user_email".to_string(), "test@example.com".to_string())
        .expect("Failed to set user email");

    let readme_blob_hash = repo
        .hash_object("./README.md".to_string(), false)
        .expect("Failed hash-object for file README.md");
    let readme_object_path = project_path
        .join(".flux/objects")
        .join(&readme_blob_hash[..2])
        .join(&readme_blob_hash[2..]);
    assert!(!readme_object_path.exists());

    repo.add("./README.md")
        .expect("Failed to add README to index");
    assert!(
        repo.index
            .map
            .get("README.md")
            .expect("Failed to find README inside index")
            == &readme_blob_hash
    );
    assert!(readme_object_path.exists());

    repo.add("./src").expect("Failed to add src to index");
    let main_hash = repo
        .hash_object("./src/main.rs".to_string(), false)
        .expect("Failed to hash src/main.rs");
    let lib_hash = repo
        .hash_object("./src/lib.rs".to_string(), false)
        .expect("Failed to hash src/lib.rs");
    assert!(
        repo.index
            .map
            .get("src/main.rs")
            .expect("Failed to find src/main.rs inside index")
            == &main_hash
    );
    assert!(
        repo.index
            .map
            .get("src/lib.rs")
            .expect("Failed to find src/lib.rs inside index")
            == &lib_hash
    );

    let main_object_path = project_path
        .join(".flux/objects")
        .join(&main_hash[..2])
        .join(&main_hash[2..]);
    assert!(main_object_path.exists());

    let commit_hash = repo
        .commit("Initial commit".to_string())
        .expect("Failed to create inital commit");
    let commit_object_path = project_path
        .join(".flux/objects")
        .join(&commit_hash[..2])
        .join(&commit_hash[2..]);
    assert!(commit_object_path.exists());

    let head_content = fs::read_to_string(".flux/HEAD").unwrap();
    assert_eq!(head_content.trim(), "ref: refs/heads/main");

    let main_ref = fs::read_to_string(".flux/refs/heads/main").unwrap();
    assert_eq!(main_ref.trim(), commit_hash);

    let commit_content = String::from_utf8(
        repo.object_store
            .retrieve_object(&commit_hash)
            .unwrap()
            .content(),
    )
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
    repo.add("./README.md")
        .expect("Failed to add README.md to index");
    let second_commit_hash = repo
        .commit("Second commit".to_string())
        .expect("Failed to create second commit");
    assert_ne!(commit_hash, second_commit_hash);

    let main_ref = fs::read_to_string(".flux/refs/heads/main").expect("Failed to read HEAD");
    assert_eq!(main_ref.trim(), second_commit_hash);

    let second_commit_content = String::from_utf8(
        repo.object_store
            .retrieve_object(&second_commit_hash)
            .unwrap()
            .content(),
    )
    .expect("Failed to read second commit content to string");

    assert!(second_commit_content.contains(&format!("parent {}", commit_hash)));
    assert!(second_commit_content.contains("Second commit"));
}

#[test]
#[serial]
fn commit_with_empty_index() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    let mut repo = Repository::init(None, false).unwrap();
    let err = repo.commit("empty".to_string()).unwrap_err();

    match err {
        flux_core::error::RepositoryError::IndexEmpty => {}
        other => panic!("expected IndexEmpty error, got: {other:?}"),
    }
    println!("{err}")
}

#[test]
#[serial]
fn commit_without_credentials() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    let mut repo = Repository::init(None, false).unwrap();
    repo.add("README.md").unwrap();

    let err = repo.commit("commit".to_string()).err().unwrap();

    match err {
        flux_core::error::RepositoryError::Context { .. } => {},
        other => panic!("unexpected error: {other:?}"),
    }

    println!("{err}")
}

#[test]
#[serial]
fn branching_test() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    let mut repo = Repository::init(None, false).expect("Failed to initalize flux repository");
    repo.set(String::from("user_name"), String::from("test"))
        .expect("Failed to set user name");
    repo.set(String::from("user_email"), String::from("test@gmail.com"))
        .expect("Failed to set user email");

    repo.add(".").expect("Failed to add changes to index");
    let first_commit_hash = repo
        .commit(String::from("First commit on branch main"))
        .unwrap();

    assert_eq!(repo.refs.head_ref().unwrap(), "refs/heads/main");
    assert_eq!(
        &repo
            .refs
            .current_branch()
            .expect("Could not read current branch name"),
        "main"
    );
    let head_content = fs::read_to_string(&repo.flux_dir.join(&repo.refs.head_ref().unwrap()))
        .expect("Could not read HEAD content");
    assert_eq!(head_content, first_commit_hash);

    repo.new_branch("feature").unwrap();
    assert!(fs::exists(&repo.flux_dir.join("refs/heads/feature")).unwrap());
    let feature_content = fs::read_to_string(&repo.flux_dir.join("refs/heads/feature")).unwrap();
    assert_eq!(feature_content, head_content);

    assert!(fs::exists(repo.work_tree.path().join("README.md")).unwrap());
    assert!(fs::exists(repo.work_tree.path().join("src/main.rs")).unwrap());
    assert!(fs::exists(repo.work_tree.path().join("src/lib.rs")).unwrap());

    repo.switch_branch("main", false).unwrap();
    assert!(fs::exists(repo.work_tree.path().join("README.md")).unwrap());
    assert!(fs::exists(repo.work_tree.path().join("src/main.rs")).unwrap());
    assert!(fs::exists(repo.work_tree.path().join("src/lib.rs")).unwrap());

    fs::write("README.md", "Added something new to README").unwrap();
    repo.add(".").expect("Failed to add changes to index");
    let second_commit_hash = repo
        .commit("Second commit on main branch".to_string())
        .expect("Failed to create the second commit on branch main");

    let main_head = fs::read_to_string(&repo.flux_dir.join("refs/heads/main")).unwrap();
    assert_eq!(second_commit_hash, main_head);
    assert_eq!(
        second_commit_hash,
        repo.refs
            .head_commit()
            .expect("Failed to read the commit HEAD points to")
    );

    repo.switch_branch("feature", false).unwrap();
    assert_eq!(
        repo.refs
            .current_branch()
            .expect("Failed to read current branch name"),
        "feature"
    );
    assert_eq!(
        first_commit_hash,
        repo.refs
            .head_commit()
            .expect("Failed to read the commit HEAD points to")
    );
    assert!(
        fs::read_to_string("./README.md")
            .unwrap()
            .contains("Read this file before running the project")
    );
    assert!(
        !fs::read_to_string("./README.md")
            .unwrap()
            .contains("Added something new to README")
    );
}



