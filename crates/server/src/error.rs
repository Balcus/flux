use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserStoreError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}
