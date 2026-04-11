use crate::{commands::command::Command, dircache::index::Index, internals::repository::Repository};

pub struct RmCommand<'a> {
    pub repo: &'a mut Repository,
    pub path: String,
}

impl<'a> RmCommand<'a> {
    pub fn new(repo: &'a mut Repository, path: String) -> Self {
        Self { repo, path }
    }
}

impl<'a> Command for RmCommand<'a> {
    fn run(&mut self) -> anyhow::Result<()> {
        let mut index = Index::new(self.repo.flux_dir.join("index"));
        index.load()?;
        index.rm(self.path.clone())?;
        Ok(())
    }
}