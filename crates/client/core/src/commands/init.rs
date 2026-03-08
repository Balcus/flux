use crate::{
    commands::command::Command,
    internals::{config::Config, index::Index, refs::Refs, work_tree::WorkTree},
};
use std::{fs, path::PathBuf};

pub struct InitCommand {
    path: Option<String>,
    force: bool,
}

impl InitCommand {
    pub fn new(path: Option<String>, force: bool) -> Self {
        Self { path, force }
    }
}

impl Command for InitCommand {
    fn run(&mut self) -> anyhow::Result<()> {
        let work_tree_path = self
            .path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let work_tree_path = work_tree_path.canonicalize()?;
        let flux_dir = work_tree_path.join(".flux");

        if flux_dir.exists() && self.force {
            fs::remove_dir_all(&flux_dir)?;
        } else if flux_dir.exists() && !self.force {
            let abs = flux_dir.canonicalize().unwrap_or_else(|_| flux_dir.clone());
            return Err(anyhow::anyhow!(
                "Repository already initialized at {}",
                abs.display()
            ));
        }

        fs::create_dir_all(&flux_dir)?;
        fs::create_dir_all(flux_dir.join("objects"))?;
        Refs::new(&flux_dir)?;
        Config::default(flux_dir.join("config"))?;
        Index::new(&flux_dir)?;
        WorkTree::new(work_tree_path);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::command::Command;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn init() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();

        InitCommand::new(Some(path.clone()), false).run().unwrap();

        let flux_dir = dir.path().join(".flux");
        assert!(flux_dir.join("config").exists());
        assert!(flux_dir.join("HEAD").exists());
        assert!(flux_dir.join("objects").exists());
        assert!(flux_dir.join("refs").exists());

        let head = fs::read_to_string(flux_dir.join("HEAD")).unwrap();
        assert_eq!(head, "ref: refs/heads/main\n");
    }

    #[test]
    fn init_already_initialized() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();

        InitCommand::new(Some(path.clone()), false).run().unwrap();
        let err = InitCommand::new(Some(path.clone()), false)
            .run()
            .unwrap_err();

        assert!(err.to_string().contains("already initialized"));
    }

    #[test]
    fn init_force_reinitializes() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();

        InitCommand::new(Some(path.clone()), false).run().unwrap();
        fs::write(dir.path().join(".flux/objects/sentinel"), "data").unwrap();

        InitCommand::new(Some(path.clone()), true).run().unwrap();

        assert!(!dir.path().join(".flux/objects/sentinel").exists());
        assert!(dir.path().join(".flux/config").exists());
        assert!(dir.path().join(".flux/HEAD").exists());
    }
}
