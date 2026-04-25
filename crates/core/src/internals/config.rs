use anyhow::Context;
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
    AccessToken,
}

impl FromStr for Field {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user_name" => Ok(Field::UserName),
            "user_email" => Ok(Field::UserEmail),
            "origin" => Ok(Field::Origin),
            "access_token" => Ok(Field::AccessToken),
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
            Field::AccessToken => "access_token",
        };
        write!(f, "{}", s)
    }
}

pub struct Credentials {
    pub user_name: String,
    pub user_email: String,
    pub access_token: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    path: PathBuf,
    pub map: HashMap<Field, Option<String>>,
}

impl Config {
    pub fn empty_map() -> HashMap<Field, Option<String>> {
        let mut map = HashMap::new();
        map.insert(Field::UserName, None);
        map.insert(Field::UserEmail, None);
        map.insert(Field::Origin, None);
        map.insert(Field::AccessToken, None);
        map
    }

    pub fn default(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();

        let mut file = File::create(&path)
            .with_context(|| format!("Failed to create '{}'.", path.display()))?;

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
        .with_context(|| format!("Failed to write '{}'.", path.display()))?;

        Ok(Self {
            path,
            map: Self::empty_map(),
        })
    }

    pub fn from(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read '{}'.", path.display()))?;

        let temp_map: HashMap<String, String> = toml::from_str(&content)
            .with_context(|| format!("Failed to parse '{}'.", path.display()))?;

        let mut map = Self::empty_map();
        for (key, value) in temp_map {
            if let Ok(field) = key.parse::<Field>() {
                map.insert(field, Some(value));
            }
        }

        Ok(Self { path, map })
    }

    pub fn set(&mut self, key: String, value: String) -> anyhow::Result<()> {
        let field = key.parse::<Field>().map_err(|_| {
            anyhow::anyhow!("The field '{key}' is unsupported by the configuration.")
        })?;

        self.map.insert(field, Some(value));

        let mut serializable_map = HashMap::new();
        for (k, v) in &self.map {
            if let Some(val) = v {
                serializable_map.insert(k.to_string(), val.clone());
            }
        }

        let toml_string = toml::to_string(&serializable_map)
            .with_context(|| format!("Failed to write '{}'.", self.path.display()))?;

        let temp_path = self.path.with_extension("tmp");
        fs::write(&temp_path, &toml_string)
            .with_context(|| format!("Failed to write '{}'.", temp_path.display()))?;
        fs::rename(&temp_path, &self.path).with_context(|| {
            format!(
                "Failed to rename '{}' to '{}'.",
                temp_path.display(),
                self.path.display()
            )
        })?;

        Ok(())
    }

    pub fn get_required(&self, field: Field) -> anyhow::Result<String> {
        self.map
            .get(&field)
            .and_then(|v| v.clone())
            .with_context(|| {
                format!("The variable '{field}' must be set, try using 'flux set {field} ...'")
            })
    }

    pub fn get_credentials(&self) -> anyhow::Result<Credentials> {
        Ok(Credentials {
            user_name: self.get_required(Field::UserName)?,
            user_email: self.get_required(Field::UserEmail)?,
            access_token: self.get("access_token")?,
        })
    }

    pub fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        let field = key.parse::<Field>().map_err(|_| {
            anyhow::anyhow!("The field '{key}' is unsupported by the configuration.")
        })?;
        Ok(self.map.get(&field).and_then(|v| v.clone()))
    }
}
