use crate::database::{commit::Commit, database::Database, tree::Tree};

pub struct TreeWalker<'a> {
    pub db: &'a Database,
}

impl<'a> TreeWalker<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn resolve_path(&self, commit_id: &str, path: &str) -> anyhow::Result<Option<String>> {
        let obj = self.db.read_object(commit_id)?;
        let commit = obj
            .as_any()
            .downcast_ref::<Commit>()
            .ok_or_else(|| anyhow::anyhow!("Not a commit"))?;

        self.resolve_path_from_tree(&commit.tree_hash, path)
    }

    fn resolve_path_from_tree(
        &self,
        tree_hash: &str,
        path: &str,
    ) -> anyhow::Result<Option<String>> {
        let mut current_tree_hash = tree_hash.to_string();
        let mut segments = path.split('/').peekable();

        while let Some(segment) = segments.next() {
            let tree_obj = self.db.read_object(&current_tree_hash)?;
            let tree = tree_obj
                .as_any()
                .downcast_ref::<Tree>()
                .ok_or_else(|| anyhow::anyhow!("Expected a tree object"))?;

            let entry = tree
                .entries()
                .iter()
                .find(|e| e.name == segment)
                .map(|e| (e.id.clone(), e.is_dir()));

            match entry {
                None => return Ok(None),

                Some((id, _)) if segments.peek().is_none() => {
                    return Ok(Some(id));
                }

                Some((id, true)) => {
                    current_tree_hash = id;
                }

                _ => return Ok(None),
            }
        }

        Ok(None)
    }
}
