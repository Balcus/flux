use crate::{
    commands::command::Command,
    internals::repository::Repository,
    status::{status_formatter::StatusFormatter, status_impl::Status},
};
use anyhow::Result;

pub struct StatusCommand<'a> {
    repo: &'a mut Repository,
}

impl<'a> StatusCommand<'a> {
    pub fn new(repo: &'a mut Repository) -> Self {
        Self { repo }
    }
}

impl<'a> Command for StatusCommand<'a> {
    fn run(&mut self) -> Result<()> {
        let status = Status::new(self.repo)?;
        let formatter = StatusFormatter::new(&status);
        formatter.print();
        Ok(())
    }
}
