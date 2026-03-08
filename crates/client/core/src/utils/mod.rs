use sha1::{Digest, Sha1};
use std::path::{Component, Path, PathBuf};

pub mod colors;
pub mod modes;

/// Computes the SHA-1 hash of the given data and returns it.
pub fn hash(data: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let object_hash = format!("{:x}", hasher.finalize());
    object_hash
}

pub fn full_path(p: impl AsRef<Path>) -> PathBuf {
    let p = p.as_ref();

    if let Ok(abs) = std::fs::canonicalize(p) {
        return abs;
    }

    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p)
    };

    let mut out = PathBuf::new();
    for c in abs.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}
