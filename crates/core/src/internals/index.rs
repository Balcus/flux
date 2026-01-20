use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Ok};
use json::JsonValue;

// TODO: i think the flush should be also after the add and delete operations as currently they probably are insdie repo
pub struct Index {
    pub map: HashMap<String, String>,
    path: PathBuf,
}

impl Index {
    pub fn new(flux_dir: &Path) -> anyhow::Result<Self> {
        let empty_obj = JsonValue::new_object();
        let json_str = empty_obj.dump();
        let path = flux_dir.join("index");
        fs::write(&path, json_str)?;
        Ok(Index {
            map: HashMap::new(),
            path,
        })
    }

    pub fn load(store_dir: &Path) -> anyhow::Result<Self> {
        let path = store_dir.join("index");

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read index file at {:?}", path))?;

        let json_obj =
            json::parse(&content).with_context(|| format!("Failed to parse JSON in {:?}", path))?;

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
