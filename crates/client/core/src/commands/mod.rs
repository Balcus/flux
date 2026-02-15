use crate::internals::repository::Repository;
use anyhow::Result;

pub mod diff;
pub mod status;

pub trait Command {
    fn run(&mut self, repo: &mut Repository) -> Result<()>;
}
