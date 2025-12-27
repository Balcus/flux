use anyhow::Context;
use serde::Deserialize;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

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
    pub fn default(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();

        let mut file = File::create(&path)
            .with_context(|| format!("Cannot initialize config file at {:?}", path))?;

        writeln!(
            file,
            "\
# Configuration file for git
# Values can be set either by modifying the file or by using the set command.
#
# user_name  =
# user_email ="
        )?;

        Ok(Self {
            path,
            user_name: None,
            user_email: None,
        })
    }

    pub fn from(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Could not read config file {:?}", path))?;

        let fields: ConfigFields =
            toml::from_str(&content).with_context(|| "Failed parsing config file")?;

        Ok(Self {
            path,
            user_name: fields.user_name,
            user_email: fields.user_email,
        })
    }

    pub fn set(&self, key: String, value: String) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)?;

        writeln!(file, r#"{key} = "{value}""#)?;
        Ok(())
    }

    pub fn get(&self) -> (String, String) {
    (
        self.user_name.clone().unwrap_or("User name is not set".to_string()),
        self.user_email.clone().unwrap_or("User email is not set".to_string())
    )
}
}
