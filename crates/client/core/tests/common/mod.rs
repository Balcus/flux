use std::{
    env, fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

pub struct WorkingDirGuard {
    original: PathBuf,
}

impl WorkingDirGuard {
    pub fn new(dir: &Path) -> anyhow::Result<Self> {
        let original = env::current_dir()?;
        env::set_current_dir(dir)?;
        Ok(Self { original })
    }
}

impl Drop for WorkingDirGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

pub fn setup_test_project() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path().to_path_buf();

    fs::write(
        project_path.join("README.md"),
        "Read this file before running the project",
    )
    .unwrap();

    fs::create_dir(project_path.join("src")).unwrap();

    fs::write(
        project_path.join("src/main.rs"),
        r#"pub fn main() { println!("{}", add(1, 2)) }"#,
    )
    .unwrap();

    fs::write(
        project_path.join("src/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i64 { a + b }",
    )
    .unwrap();

    (temp, project_path)
}
