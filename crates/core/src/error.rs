use hex::FromHexError;
use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IoError {
    #[error("failed to read {}. {source}.", path.display())]
    Read { path: PathBuf, source: io::Error },

    #[error("failed to write {}. {source}.", path.display())]
    Write { path: PathBuf, source: io::Error },

    #[error("failed to create {}. {source}.", path.display())]
    Create { path: PathBuf, source: io::Error },

    #[error("failed to delete {}. {source}.", path.display())]
    Delete { path: PathBuf, source: io::Error },

    #[error("missing path {path}.")]
    Missing { path: PathBuf },

    #[error("failed rename from: {} to: {}. {source}.", from.display(), to.display())]
    Rename {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },

    #[error("failed to open file {}. {source}", path.display())]
    Open { path: PathBuf, source: io::Error },
}

#[derive(Error, Debug)]
#[error("failed to parse '{}'. {source}", path.display())]
#[non_exhaustive]
pub struct ParseError {
    path: PathBuf,
    #[source]
    pub source: json::Error,
}

impl ParseError {
    pub fn new(path: PathBuf, source: json::Error) -> Self {
        Self { path, source }
    }
}

#[derive(Error, Debug)]
pub enum ObjectError {
    #[error("invalid object format: {hash} at {path}.")]
    InvalidFormat { path: PathBuf, hash: String },

    #[error("unsupported object type: {object_type}.")]
    Unsupported { object_type: String },

    #[error("size mismatch: {hash} at {path}: expected {expected}, got {got}.")]
    SizeMismatch {
        path: PathBuf,
        hash: String,
        expected: usize,
        got: usize,
    },
}

#[derive(Debug, Error)]
pub enum ObjectStoreError {
    #[error(transparent)]
    Object(#[from] ObjectError),

    #[error(transparent)]
    Io(#[from] IoError),
}

#[derive(Debug, Error)]
pub enum IndexError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Parse(#[from] ParseError),
}

#[derive(Debug, Error)]
pub enum WorkTreeError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    ObjectStore(#[from] ObjectStoreError),

    #[error("object downcast error, expected type: {expected}.")]
    Downcast { expected: &'static str },

    #[error("invalid object hash : {hash}. {source}.")]
    InvalidHash { hash: String, source: FromHexError },
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error("failed to create toml from string. {0}")]
    TomlFromString(#[from] toml::de::Error),

    #[error("the variable {0} must be set, try the set command to do so")]
    NotSet(&'static str),
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("repositroy already initialized at {0}, if this is intentional use the --force flag.")]
    AlreadyInitialized(PathBuf),

    #[error("{context}. {source}")]
    Context {
        context: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("cannot use object {hash} as commit root, object is not a tree")]
    CommitRoot {
        hash: String
    },

    #[error("nothing to commit, index is empty")]
    IndexEmpty
}

impl RepositoryError {
    pub fn with_context<E>(context: &'static str, err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        RepositoryError::Context {
            context,
            source: Box::new(err),
        }
    }
}
