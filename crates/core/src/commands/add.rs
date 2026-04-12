use crate::commands::command::Command;
use crate::database::blob::Blob;
use crate::database::database::Database;
use crate::database::object::Object;
use crate::dircache::index::Index;
use crate::internals::repository::Repository;
use std::path::{Path, PathBuf};

pub struct AddCommand<'a> {
    pub repo: &'a mut Repository,
    pub path: PathBuf,
}

impl<'a> AddCommand<'a> {
    pub fn new(repo: &'a mut Repository, path: PathBuf) -> Self {
        Self { repo, path }
    }

    fn add_path(&self, path: &Path, index: &mut Index, db: &Database) -> anyhow::Result<()> {
        if !path.exists() {
            let relative_path = path
                .strip_prefix(self.repo.work_tree.path())?
                .to_str()
                .unwrap()
                .to_string();
            index.rm(relative_path)?;
            return Ok(());
        }

        if path.is_file() {
            let data = self.repo.work_tree.read_file(path)?;
            let blob = Blob::from_bytes(data);
            let id = blob.id().clone();
            db.store(Box::from(blob))?;

            let relative_path = path
                .strip_prefix(self.repo.work_tree.path())?
                .to_str()
                .unwrap()
                .to_string();
            let stat = path.metadata()?;
            index.add(relative_path, id, stat)?;
        } else if path.is_dir() {
            if path.ends_with(".flux") {
                return Ok(());
            }
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let child = entry.path();
                if child.ends_with(".flux") {
                    continue;
                }
                self.add_path(&child, index, db)?;
            }
        }
        Ok(())
    }
}

impl<'a> Command for AddCommand<'a> {
    fn run(&mut self) -> anyhow::Result<()> {
        let full_path = self.repo.work_tree.path().join(&self.path);
        let mut index = Index::new(self.repo.flux_dir.join("index"));
        index.load()?;
        let db = Database::open(self.repo.flux_dir.clone());

        self.add_path(&full_path, &mut index, &db)?;

        if full_path.is_dir() {
            let prefix = if self.path == PathBuf::from(".") {
                String::new()
            } else {
                format!("{}/", self.path.to_str().unwrap().trim_end_matches('/'))
            };

            let indexed_paths: Vec<String> = index
                .entries
                .keys()
                .filter(|(k, stage)| *stage == 0 && (prefix.is_empty() || k.starts_with(&prefix)))
                .map(|(k, _)| k.clone())
                .collect();

            for indexed_path in indexed_paths {
                let file_full_path = self.repo.work_tree.path().join(&indexed_path);
                if !file_full_path.exists() {
                    index.rm(indexed_path)?;
                }
            }
        }

        index.write_updates()?;
        Ok(())
    }
}
