use crate::error;
use serde::Deserialize;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

// TODO: only allow the set of preset fileds and on get return a struct for them insted of tuple
#[derive(Deserialize)]
pub struct ConfigFields {
    user_name: Option<String>,
    user_email: Option<String>,
}

pub struct Config {
    path: PathBuf,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
}

impl Config {
    pub fn default(path: impl Into<PathBuf>) -> Result<Self, error::ConfigError> {
        let path = path.into();

        let mut file = File::create(&path).map_err(|e| error::IoError::Create {
            path: path.clone(),
            source: e,
        })?;

        writeln!(
            file,
            "\
# Configuration file for git
# Values can be set either by modifying the file or by using the set command.
#
# user_name  =
# user_email ="
        )
        .map_err(|e| error::IoError::Write {
            path: path.clone(),
            source: e,
        })?;

        Ok(Self {
            path,
            user_name: None,
            user_email: None,
        })
    }

    pub fn from(path: impl Into<PathBuf>) -> Result<Self, error::ConfigError> {
        let path = path.into();

        let content = fs::read_to_string(&path).map_err(|e| error::IoError::Read {
            path: path.clone(),
            source: e,
        })?;

        let fields: ConfigFields =
            toml::from_str(&content).map_err(|e| error::ConfigError::TomlFromString(e))?;

        Ok(Self {
            path,
            user_name: fields.user_name,
            user_email: fields.user_email,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<(), error::ConfigError> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| error::IoError::Open {
                path: self.path.clone(),
                source: e,
            })?;

        writeln!(file, r#"{key} = "{value}""#).map_err(|e| error::IoError::Write {
            path: self.path.clone(),
            source: e,
        })?;

        match key.as_str() {
            "user_name" => self.user_name = Some(value),
            "user_email" => self.user_email = Some(value),
            _ => {}
        }

        Ok(())
    }

    // this needs to change soon
    pub fn get(&self) -> Result<(String, String), error::ConfigError> {
        let user_name = self
            .user_name
            .clone()
            .ok_or_else(|| error::ConfigError::NotSet("user_name"))?;
        let user_email = self
            .user_email
            .clone()
            .ok_or_else(|| error::ConfigError::NotSet("user_email"))?;

        Ok((user_name.clone(), user_email.clone()))
    }
}
