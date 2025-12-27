use flux::{commands, repo::repository::Repository};
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

    Repository::init(None, false).unwrap();

    assert!(project_path.join(".flux/config").exists());
    assert!(project_path.join(".flux/HEAD").exists());
    assert!(project_path.join(".flux/objects").exists());
    assert!(project_path.join(".flux/refs").exists());

    let head = fs::read_to_string(".flux/HEAD").unwrap();
    assert_eq!(head, "ref: refs/heads/main\n");
}

#[test]
#[serial]
fn set_test() {
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    Repository::init(None, false).unwrap();

    commands::set("user_name".to_string(), "user".to_string()).unwrap();
    commands::set("user_email".to_string(), "user@gmail.com".to_string()).unwrap();

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

    // file hashing
    let my_hash = commands::hash_object("README.md".to_string(), false).unwrap();
    let git_hash = common::git_hash_object("README.md").unwrap();
    assert_eq!(my_hash, git_hash);
    let object_path = project_path
        .join(".flux/objects")
        .join(&git_hash[..2])
        .join(&git_hash[2..]);
    assert!(!object_path.exists());
    let _ = commands::hash_object("README.md".to_string(), true).unwrap();
    assert!(object_path.exists());

    // dir hashing (git does not support hashing directories directly)
    assert!(project_path.join("src").exists());
    let my_hash = commands::hash_object("src".to_string(), true).unwrap();
    assert_eq!(my_hash, "ac715a76cc52acc719def812525f6ae57b4770a9");
}

#[test]
#[serial]
fn commit() {
    // TODO:
    let (_temp, project_path) = common::setup_test_project();
    let _guard = common::WorkingDirGuard::new(&project_path).unwrap();

    Repository::init(None, false).unwrap();
}
