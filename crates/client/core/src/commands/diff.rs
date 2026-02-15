use crate::{
    commands::Command,
    internals::repository::Repository,
    status::status_impl::Status,
};
use anyhow::Result;

pub struct DiffCommand;

impl DiffCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for DiffCommand {
    fn run(&mut self, repo: &mut Repository) -> Result<()> {
        let status = Status::new(repo)?;
        Ok(())
    }
}
