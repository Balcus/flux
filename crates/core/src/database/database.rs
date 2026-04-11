use crate::database::{
    blob::Blob, commit::Commit, disk_utils::DiskUtils, object::Object, object_parser,
    object_type::ObjectType, tree::Tree,
};
use std::{collections::HashMap, path::PathBuf};

const OBJECTS_PATH: &str = "objects";

pub struct Database {
    pub path: PathBuf,
    pub cache: HashMap<String, Box<dyn Object>>,
    pub disk_utils: DiskUtils,
}

impl Database {
    /// expects the path to the flux directory as input
    pub fn open(path: PathBuf) -> Self {
        Self {
            disk_utils: DiskUtils::new(path.clone().join(OBJECTS_PATH)),
            path,
            cache: HashMap::new(),
        }
    }

    pub fn store(&self, object: Box<dyn Object>) -> anyhow::Result<()> {
        let serialized = object.serialize();
        self.disk_utils.write_object(&object.id(), &serialized)?;
        Ok(())
    }

    pub fn read_object(&self, id: &str) -> anyhow::Result<Box<dyn Object>> {
        let decomp_bytes = self.disk_utils.read_raw(id)?;
        let raw_obj = object_parser::parse(decomp_bytes)?;
        let obj: Box<dyn Object> = match raw_obj.object_type {
            ObjectType::Blob => Box::new(Blob::from_bytes(raw_obj.data)),
            ObjectType::Tree => Box::new(Tree::from_content(raw_obj.data)),
            ObjectType::Commit => Box::new(Commit::from_content(raw_obj.data)),
            ObjectType::Tag => anyhow::bail!("Object type tag currently unsupported."),
        };
        Ok(obj)
    }

    pub fn commit_to_map(&self, commit_id: String) -> anyhow::Result<HashMap<String, String>> {
        let obj = self.read_object(&commit_id)?;

        let commit = obj
            .as_any()
            .downcast_ref::<Commit>()
            .ok_or_else(|| anyhow::anyhow!("Provided object is not a commit"))?;

        let obj = self.read_object(&commit.tree_hash)?;
        let tree = obj
            .as_any()
            .downcast_ref::<Tree>()
            .ok_or_else(|| anyhow::anyhow!("Provided object is not a commit"))?;

        self.tree_to_map(tree, "")
    }

    fn tree_to_map(&self, tree: &Tree, prefix: &str) -> anyhow::Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        for entry in tree.entries() {
            let full_path = if prefix.is_empty() {
                entry.name.clone()
            } else {
                format!("{}{}", prefix, entry.name)
            };

            if entry.is_dir() {
                let obj = self.read_object(&entry.id)?;
                let subtree = obj
                    .as_any()
                    .downcast_ref::<Tree>()
                    .ok_or_else(|| anyhow::anyhow!("Error"))?;
                let submap = self.tree_to_map(subtree, &format!("{}/", full_path))?;
                map.extend(submap);
            } else {
                map.insert(full_path, entry.id);
            }
        }
        Ok(map)
    }

    pub fn short_id(&self, id: &String) -> String {
        if id.len() < 7 {
            return id.clone();
        }
        String::from(&id[0..7])
    }
}
