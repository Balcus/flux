use hex::FromHexError;
use std::{
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::utils;

#[derive(Error, Debug)]
pub enum IoError {
    #[error("Failed to read '{}'. {source}.", path.display())]
    Read { path: PathBuf, source: io::Error },

    #[error("Failed to write '{}'. {source}.", path.display())]
    Write { path: PathBuf, source: io::Error },

    #[error("Failed to create '{}'. {source}.", path.display())]
    Create { path: PathBuf, source: io::Error },

    #[error("Failed to delete '{}'. {source}.", path.display())]
    Delete { path: PathBuf, source: io::Error },

    #[error("Missing required path '{}'.", path.display())]
    Missing { path: PathBuf },

    #[error("Failed rename from: '{}' to: '{}'. {source}.", from.display(), to.display())]
    Rename {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },

    #[error("Failed to open file '{}'. {source}", path.display())]
    Open { path: PathBuf, source: io::Error },

    #[error("Failed to read metadata for '{}'. {source}", path.display())]
    Metadata { path: PathBuf, source: io::Error },
}

impl IoError {
    pub fn read_error(path: impl AsRef<Path>, source: io::Error) -> Self {
        IoError::Read {
            path: utils::full_path(path),
            source,
        }
    }

    pub fn write_error(path: impl AsRef<Path>, source: io::Error) -> Self {
        IoError::Write {
            path: utils::full_path(path),
            source,
        }
    }

    pub fn create_error(path: impl AsRef<Path>, source: io::Error) -> Self {
        IoError::Create {
            path: utils::full_path(path),
            source,
        }
    }

    pub fn delete_error(path: impl AsRef<Path>, source: io::Error) -> Self {
        IoError::Delete {
            path: utils::full_path(path),
            source,
        }
    }

    pub fn missing_error(path: impl AsRef<Path>) -> Self {
        IoError::Missing {
            path: utils::full_path(path),
        }
    }

    pub fn metadata_error(path: impl AsRef<Path>, source: io::Error) -> Self {
        IoError::Metadata {
            path: utils::full_path(path),
            source,
        }
    }

    pub fn rename_error(from: impl AsRef<Path>, to: impl AsRef<Path>, source: io::Error) -> Self {
        IoError::Rename {
            from: utils::full_path(from),
            to: utils::full_path(to),
            source,
        }
    }
}

#[derive(Error, Debug)]
#[error("Failed to parse '{}'. {source}", path.display())]
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
    #[error("Invalid format for object {hash} at '{path}'.")]
    InvalidFormat { path: PathBuf, hash: String },

    #[error("Unsupported object type: '{object_type}'.")]
    Unsupported { object_type: String },

    #[error("Size mismatch for object {hash} at '{path}': expected {expected}, got {got}.")]
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

    #[error("Object downcast error, expected type: '{expected}'.")]
    Downcast { expected: &'static str },
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

    #[error("Object downcast error, expected type: '{expected}'.")]
    Downcast { expected: &'static str },

    #[error("Invalid object hash: {hash}. {source}.")]
    InvalidHash { hash: String, source: FromHexError },
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error("Failed to create toml from string. {0}")]
    TomlFromString(#[from] toml::de::Error),

    #[error("The variable {0} must be set, try using 'flux set {0} ...'")]
    NotSet(String),

    #[error("The field {0} is unsupported by the configuration.")]
    UnsupportedField(String),
}

#[derive(Debug, Error)]
pub enum RefsError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error("Invalid head format: {head}.")]
    InvalidHead { head: String },

    #[error("Branch '{0}' already exists.")]
    BranchAlreadyExists(String),

    #[error("Branch '{0}' does not exist.")]
    MissingBranch(String),

    #[error("Cannot delete the current branch '{0}'. Switch to a different branch and try again.")]
    DeleteCurrentBranch(String),
}

#[derive(Debug, Error)]
pub enum GrpcClientError {
    #[error("Failed to connect to remote repository at '{url}'. {source}")]
    ConnectRemote {
        url: String,
        #[source]
        source: tonic::transport::Error,
    },

    #[error("Failed push to remote repoository. {0}")]
    Push(#[source] tonic::Status),

    #[error("Failed to parse url: '{url}'.")]
    Url {
        url: String,
        #[source]
        source: Option<url::ParseError>,
    },

    #[error("Failed to clone repository.{0}")]
    Clone(#[source] tonic::Status),

    #[error("Failed authentication for remote server.")]
    Auth(#[source] tonic::Status)
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error("Repositroy already initialized at {0}, if this is intentional use the --force flag.")]
    AlreadyInitialized(PathBuf),

    #[error("{context}. {source}")]
    Other {
        context: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Cannot use object {hash} as commit root, object is not a tree.")]
    CommitRoot { hash: String },

    #[error("Nothing to commit, index is empty.")]
    IndexEmpty,

    #[error("Repository not initialized at: '{0}'. Run 'flux init {0}' and try again.")]
    NotRepository(PathBuf),

    #[error("There was an error trying to operate on path: '{}'.", path.display())]
    PathName { path: PathBuf },

    #[error("Repository has uncommited changes, commit them and try again or use the --force flag")]
    UncommitedChanges,

    #[error("User credentials are not set. {0}")]
    Credentials(#[source] ConfigError),

    #[error(transparent)]
    ObjectStore(#[from] ObjectStoreError),

    #[error(transparent)]
    Configuration(#[from] ConfigError),

    #[error(transparent)]
    Refs(#[from] RefsError),

    #[error(transparent)]
    WorkTree(#[from] WorkTreeError),

    #[error(transparent)]
    IndexError(#[from] IndexError),

    #[error(
        "Missing origin for remote repository. Specify it in the push command like 'flux push http://originurl' or set it in the config with 'flux set origin http://originurl'"
    )]
    MissingOrigin(),

    #[error("Failed to archive flux repository")]
    Archive(#[from] std::io::Error),

    #[error(transparent)]
    Grpc(#[from] GrpcClientError),

    #[error("Missing access token from remote server. Try running flux auth and try again.")]
    MissingToken
}

impl RepositoryError {
    pub fn from<E>(context: &'static str, err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        RepositoryError::Other {
            context,
            source: Box::new(err),
        }
    }
}
