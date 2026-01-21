use crate::error::{self, IoOperation};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use json::JsonValue;

// TODO: i think the flush should be also after the add and delete operations as currently they probably are insdie repo
#[derive(Debug)]
pub struct Index {
    pub map: HashMap<String, String>,
    path: PathBuf,
}

impl Index {
    pub fn new(flux_dir: &Path) -> Result<Self, error::IoError> {
        let json_str = JsonValue::new_object().dump();
        let path = flux_dir.join("index");

        fs::write(&path, json_str)
            .map_err(|e| error::IoError::new(IoOperation::Write, path.clone(), e))?;

        Ok(Index {
            map: HashMap::new(),
            path,
        })
    }

    pub fn load(store_dir: &Path) -> Result<Self, error::FluxError> {
        let path = store_dir.join("index");

        let content = fs::read_to_string(&path)
            .map_err(|e| error::IoError::new(IoOperation::Read, path.clone(), e))?;

        let json_obj =
            json::parse(&content).map_err(|e| error::ParseError::new(path.clone(), e))?;

        let mut map = HashMap::new();
        if let JsonValue::Object(obj) = json_obj {
            for (key, value) in obj.iter() {
                if let JsonValue::String(s) = value {
                    map.insert(key.to_string(), s.to_string());
                }
            }
        }
        Ok(Index { map, path })
    }

    pub fn flush(&self) -> anyhow::Result<()> {
        let mut json_obj = JsonValue::new_object();
        for (key, value) in &self.map {
            json_obj[key] = value.clone().into();
        }
        std::fs::write(&self.path, json_obj.dump())?;

        Ok(())
    }

    pub fn add(&mut self, path: String, hash: String) -> anyhow::Result<()> {
        self.map.insert(path, hash);

        Ok(())
    }

    pub fn remove(&mut self, path: String) -> anyhow::Result<()> {
        self.map.remove(&path);

        Ok(())
    }

    pub fn clear(&mut self) -> anyhow::Result<()> {
        self.map.clear();
        self.flush()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn init_fail() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("not_a_dir");
        File::create(&file_path).expect("Failed to create file");

        // call new method with file instead of dir, should throw io exception
        let result = Index::new(&file_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.operation, IoOperation::Write);
    }

    #[test]
    // continue test-case
    fn load_fail() {
        let dir = tempdir().unwrap();
        fs::create_dir(".flux").unwrap();
        let result = Index::load(dir.path());
        assert!(result.is_err());
        // let err = result.unwrap_err();
        // File::create(".flux/index").unwrap();
    }
}
