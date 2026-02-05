use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserStoreError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error("Email '{0}' is already registered on the server.")]
    EmailAlredyRegistered(String),
}