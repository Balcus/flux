use crate::error;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fmt, fs};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Field {
    UserName,
    UserEmail,
    Origin,
}

impl FromStr for Field {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user_name" => Ok(Field::UserName),
            "user_email" => Ok(Field::UserEmail),
            "origin" => Ok(Field::Origin),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Field::UserName => "user_name",
            Field::UserEmail => "user_email",
            Field::Origin => "origin",
        };
        write!(f, "{}", s)
    }
}

pub struct Credentials {
    pub user_name: String,
    pub user_email: String,
}

#[derive(Debug)]
pub struct Config {
    path: PathBuf,
    pub map: HashMap<Field, Option<String>>,
}

impl Config {
    pub fn empty_map() -> HashMap<Field, Option<String>> {
        let mut map = HashMap::new();
        for field in [Field::UserName, Field::UserEmail, Field::Origin] {
            map.insert(field, None);
        }
        map
    }

    pub fn default(path: impl Into<PathBuf>) -> Result<Self, error::ConfigError> {
        let path = path.into();

        let mut file = File::create(&path).map_err(|e| error::IoError::Create {
            path: path.clone(),
            source: e,
        })?;

        writeln!(
            file,
            "\
# Configuration file for flux
# Values can be set either by directly modifying the file or by using the set command.
#
# user_name  =
# user_email =
# origin ="
        )
        .map_err(|e| error::IoError::Write {
            path: path.clone(),
            source: e,
        })?;

        Ok(Self {
            path,
            map: Self::empty_map(),
        })
    }

    pub fn from(path: impl Into<PathBuf>) -> Result<Self, error::ConfigError> {
        let path = path.into();

        let content = fs::read_to_string(&path).map_err(|e| error::IoError::Read {
            path: path.clone(),
            source: e,
        })?;

        let temp_map: HashMap<String, String> =
            toml::from_str(&content).map_err(error::ConfigError::TomlFromString)?;

        let mut map = Self::empty_map();

        for (key, value) in temp_map {
            if let Ok(field) = key.parse::<Field>() {
                map.insert(field, Some(value));
            }
        }

        Ok(Self { path, map })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<(), error::ConfigError> {
        let field = key
            .parse::<Field>()
            .map_err(|_| error::ConfigError::UnsupportedField(key.clone()))?;

        self.map.insert(field, Some(value));

        let mut serializable_map = std::collections::HashMap::new();
        for (k, v) in &self.map {
            if let Some(val) = v {
                serializable_map.insert(k.to_string(), val.clone());
            }
        }

        let toml_string =
            toml::to_string(&serializable_map).map_err(|e| error::IoError::Write {
                path: self.path.clone(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            })?;

        let temp_path = self.path.with_extension("tmp");
        std::fs::write(&temp_path, &toml_string).map_err(|e| error::IoError::Write {
            path: temp_path.clone(),
            source: e,
        })?;

        std::fs::rename(&temp_path, &self.path).map_err(|e| error::IoError::Write {
            path: self.path.clone(),
            source: e,
        })?;

        Ok(())
    }

    fn get_required(&self, field: Field) -> Result<String, error::ConfigError> {
        self.map
            .get(&field)
            .and_then(|v| v.clone())
            .ok_or_else(|| error::ConfigError::NotSet(field.to_string()))
    }

    pub fn get_credential(&self) -> Result<Credentials, error::ConfigError> {
        Ok(Credentials {
            user_name: self.get_required(Field::UserName)?,
            user_email: self.get_required(Field::UserEmail)?,
        })
    }

    pub fn get(&self, key: &str) -> Result<String, error::ConfigError> {
        let field = &key
            .parse::<Field>()
            .map_err(|_| error::ConfigError::UnsupportedField(key.to_string()))?;

        let val = self
            .map
            .get(field)
            .and_then(|v| v.clone())
            .ok_or_else(|| error::ConfigError::NotSet(key.to_string()))?;

        Ok(val)
    }
}
