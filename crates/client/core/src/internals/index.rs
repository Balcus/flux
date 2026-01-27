use crate::error;
use json::JsonValue;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Index {
    pub map: HashMap<String, String>,
    path: PathBuf,
}

impl Index {
    pub fn new(flux_dir: &Path) -> Result<Self, error::IndexError> {
        let json_str = JsonValue::new_object().dump();
        let path = flux_dir.join("index");

        fs::write(&path, json_str).map_err(|e| error::IoError::Write {
            path: path.clone(),
            source: e,
        })?;

        Ok(Index {
            map: HashMap::new(),
            path,
        })
    }

    pub fn load(store_dir: &Path) -> Result<Self, error::IndexError> {
        let path = store_dir.join("index");

        if !path.exists() {
            return Err(error::IoError::Missing { path: path.clone() }.into());
        }

        let content = fs::read_to_string(&path).map_err(|e| error::IoError::Read {
            path: path.clone(),
            source: e,
        })?;

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

    pub fn flush(&self) -> Result<(), error::IndexError> {
        let mut json_obj = JsonValue::new_object();
        for (key, value) in &self.map {
            json_obj[key] = value.clone().into();
        }
        std::fs::write(&self.path, json_obj.dump()).map_err(|e| error::IoError::Write {
            path: self.path.clone(),
            source: e,
        })?;
        Ok(())
    }

    pub fn add(&mut self, path: String, hash: String) -> Result<(), error::IndexError> {
        self.map.insert(path, hash);
        self.flush()
    }

    pub fn remove(&mut self, path: &str) -> Result<bool, error::IndexError> {
        let res = self.map.remove(path);
        self.flush()?;
        Ok(res.is_some())
    }

    pub fn clear(&mut self) -> Result<(), error::IndexError> {
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
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("not_a_dir");
        File::create(&file_path).unwrap();

        let result = Index::new(&file_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, error::IndexError::Io(error::IoError::Write { .. })));

        println!("{err}")
    }

    #[test]
    fn load_fail() {
        let dir = tempdir().unwrap();
        let flux_dir = dir.path().join(".flux");
        if flux_dir.exists() {
            fs::remove_dir(&flux_dir).unwrap();
        }
        fs::create_dir(&flux_dir).unwrap();

        let res = Index::load(&flux_dir);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(err, error::IndexError::Io(error::IoError::Missing { .. })));

        println!("{err}");

        File::create(flux_dir.join("index")).unwrap();
        fs::write(flux_dir.join("index"), "{").unwrap();

        let res = Index::load(&flux_dir);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(err, error::IndexError::Parse(..)));

        println!("{err}");
    }
}
