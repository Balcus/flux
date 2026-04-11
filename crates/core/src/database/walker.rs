use crate::database::{commit::Commit, database::Database, tree::Tree};

/// Utility struct meant to query the tree structure of the object database
pub struct Walker<'a> {
    pub db: &'a Database,
}

impl<'a> Walker<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn file_hash_from_commit(
        &self,
        path: &str,
        commit_id: &str,
    ) -> anyhow::Result<Option<String>> {
        let obj = self.db.read_object(commit_id)?;
        let commit = obj
            .as_any()
            .downcast_ref::<Commit>()
            .ok_or_else(|| anyhow::anyhow!("Not a commit"))?;

        let mut current_tree_hash = commit.tree_hash.clone();

        let mut segments = path.split('/').peekable();
        while let Some(segment) = segments.next() {
            let tree_obj = self.db.read_object(&current_tree_hash)?;
            let tree = tree_obj
                .as_any()
                .downcast_ref::<Tree>()
                .ok_or_else(|| anyhow::anyhow!("Expected a tree object"))?;

            let entries = tree.entries();
            let entry = entries.iter().find(|e| e.name == segment);

            match entry {
                None => return Ok(None),
                Some(e) if segments.peek().is_none() => return Ok(Some(e.id.clone())),
                Some(e) if e.is_dir() => {
                    current_tree_hash = e.id.clone();
                }
                _ => return Ok(None),
            }
        }

        Ok(None)
    }
}
