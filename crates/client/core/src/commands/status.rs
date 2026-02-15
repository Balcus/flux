use anyhow::Result;
use crate::{
    commands::Command,
    internals::repository::Repository,
    status::{status_formatter::StatusFormatter, status_impl::Status},
};

pub struct StatusCommand;

impl StatusCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for StatusCommand {
    fn run(&mut self, repo: &mut Repository) -> Result<()> {
        let status = Status::new(repo)?;
        let formatter = StatusFormatter::new(&status);
        formatter.print();
        Ok(())
    }
}
