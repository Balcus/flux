use std::{fmt, io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub enum IoOperation {
    Read,
    Write,
    Create,
    Delete
}

impl fmt::Display for IoOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Create => write!(f, "create"),
            Self::Delete => write!(f, "delete"),
        }
    }
}

#[derive(Error, Debug)]
#[error("Failed to {operation} '{}', inner: {source}", path.display())]
#[non_exhaustive]
pub struct IoError {
    pub operation: IoOperation,
    path: PathBuf,
    #[source]
    pub source: io::Error
}

impl IoError {
    pub fn new(operation: IoOperation, path: PathBuf, source: io::Error) -> Self {
        Self {
            operation,
            path,
            source
        }
    }
}

#[derive(Error, Debug)]
#[error("Failed to parse '{}', inner: {source}", path.display())]
#[non_exhaustive]
pub struct ParseError {
    path: PathBuf,
    #[source]
    pub source: json::Error
}

impl ParseError {
    pub fn new(path: PathBuf, source: json::Error) -> Self {
        Self {
            path,
            source
        }
    }
}


#[derive(Error, Debug)]
pub enum FluxError {
    #[error("io error")]
    Io(#[from] IoError),
    
    #[error("parse error")]
    Parse(#[from] ParseError),
}

