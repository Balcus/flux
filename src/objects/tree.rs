use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::io::Write;

pub struct Tree;

impl Tree {
    pub fn hash_object(content: Vec<u8>) -> Result<(String, Vec<u8>), anyhow::Error> {
        let header = format!("tree {}\0", content.len());
        let mut store = Vec::new();
        store.extend_from_slice(header.as_bytes());
        store.extend_from_slice(&content);

        let mut hasher = Sha1::new();
        hasher.update(&store);
        let hash = hex::encode(hasher.finalize());

        let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(&store)?;
        let compressed = encoder.finish()?;

        Ok((hash, compressed))
    }
}
