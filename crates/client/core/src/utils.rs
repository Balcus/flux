use crate::objects::object_type::ObjectType;
use flate2::{Compression, bufread::ZlibDecoder, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use std::{
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

pub struct GenericObject {
    pub object_type: ObjectType,
    pub size: usize,
    pub decompressed_content: Vec<u8>,
}

/// Decompresses zlib-compressed data using the DEFLATE algorithm.
/// Takes compressed bytes and returns the original uncompressed data
pub fn decompress(compressed: Vec<u8>) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut result = Vec::new();
    decoder
        .read_to_end(&mut result)
        .expect("Failed to decompress data");
    result
}

/// Computes the SHA-1 hash of the given data and returns it.
pub fn hash(data: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let object_hash = format!("{:x}", hasher.finalize());
    object_hash
}

/// Compresses data using zlib compression with default compression level.
/// Returns the compressed bytes.
pub fn compress(data: &Vec<u8>) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).expect("Failed to compress data");

    encoder.finish().expect("Failed to compress data")
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
