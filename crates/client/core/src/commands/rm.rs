use std::path::PathBuf;
use crate::{commands::command::Command, internals::repository::Repository};

pub struct RmCommand<'a> {
    pub repo: &'a mut Repository,
    pub path: PathBuf,
}

impl<'a> Command for RmCommand<'a> {
    fn run(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}