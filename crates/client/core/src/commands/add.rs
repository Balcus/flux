use crate::database::blob::Blob;
use crate::database::database::Database;
use crate::database::object::Object;
use crate::dircache::index::Index;
use crate::internals::repository::Repository;
use crate::internals::work_tree::WorkTree;
use std::path::PathBuf;

pub struct AddCommand {
    pub root_path: String,
    pub path: PathBuf,
}

impl AddCommand {
    pub fn run(&mut self) -> anyhow::Result<()> {
        let repo = Repository::open(Some(self.root_path.clone()))?;
        let work_tree = WorkTree::new(PathBuf::from(&self.root_path));
        let db = Database::open(repo.flux_dir.clone());

        let mut index = Index::new(repo.flux_dir.join("index"));

        let data = work_tree.read_file(&self.path)?;
        let stat = work_tree.stat_file(&self.path)?.unwrap();

        let blob = Blob::from_bytes(data);
        let id = blob.id().clone();
        db.store(Box::from(blob))?;

        let relative_path = self.path.to_str().unwrap().to_string();
        index.add(relative_path, id, stat)?;

        index.write_updates()?;
        Ok(())
    }
}
