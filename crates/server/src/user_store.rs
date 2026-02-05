use crate::error;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, path::PathBuf};

pub type Result<T> = std::result::Result<T, error::UserStoreError>;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserStore {
    #[serde(skip)]
    pub path: PathBuf,
    #[serde(flatten)]
    pub map: HashMap<String, UserStoreEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserStoreEntry {
    pub user_name: String,
    pub access_token: String,
}

impl UserStore {
    pub fn new(path_str: Option<String>) -> Result<Self> {
        let path = PathBuf::from(path_str.unwrap_or_else(|| "user_store.json".to_string()));
        let store = Self {
            path,
            map: HashMap::new(),
        };
        store.save()?;
        Ok(store)
    }

    pub fn open(path_str: Option<String>) -> Result<Self> {
        let path = PathBuf::from(path_str.unwrap_or_else(|| "user_store.json".to_string()));
        let file = File::open(&path)?;
        let mut store: Self = serde_json::from_reader(file)?;
        store.path = path;
        Ok(store)
    }

    pub fn save(&self) -> Result<()> {
        let file = File::create(&self.path)?;
        Ok(serde_json::to_writer_pretty(file, self)?)
    }

    pub async fn add_user(
        &mut self,
        user_name: String,
        user_email: String,
        access_token: String,
    ) -> Result<()> {
        if self.map.contains_key(&user_email) {
            return Err(error::UserStoreError::EmailAlredyRegistered(user_email));
        }
        let entry = UserStoreEntry {
            user_name,
            access_token,
        };

        self.map.insert(user_email, entry);
        self.save()?;
        Ok(())
    }

    pub async fn is_token_valid(
        &self,
        user_name: String,
        user_email: String,
        access_token: String,
    ) -> bool {
        if let Some(entry) = self.map.get(&user_email) {
            return entry.access_token == access_token && entry.user_name == user_name;
        } else {
            false
        }
    }
}
