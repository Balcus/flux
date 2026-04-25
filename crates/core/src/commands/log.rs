use crate::{
    commands::command::Command,
    database::{commit::Commit, database::Database},
    internals::repository::Repository,
};

pub struct LogCommand<'a> {
    repo: &'a Repository,
    db: Database,
}

impl<'a> LogCommand<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self {
            repo,
            db: Database::open(repo.flux_dir.clone()),
        }
    }
}

impl<'a> Command for LogCommand<'a> {
    type Output = ();
    
    fn run(&mut self) -> anyhow::Result<()> {
        let mut current_hash = self.repo.refs.head_commit().ok().filter(|s| !s.is_empty());

        while let Some(hash) = current_hash {
            let obj = self.db.read_object(&hash).unwrap();
            println!("{}", obj);
            let current = self.db.read_object(&hash).unwrap();
            if let Some(commit) = current.as_any().downcast_ref::<Commit>() {
                current_hash = commit.parent_hash().map(String::from);
            } else {
                break;
            }
        }

        Ok(())
    }
}
