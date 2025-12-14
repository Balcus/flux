use std::io::Write;
use anyhow::Context;

pub struct Blob;

impl Blob {
    pub fn cat_file(_size: usize, content: Vec<u8>) -> Result<String, anyhow::Error> {
        let content_str = String::from_utf8(content)
            .with_context(|| "Could not parse blob content")?;
        Ok(content_str)
    }

    pub fn hash_object(content: Vec<u8>) -> Result<(String, Vec<u8>), anyhow::Error> {
        let header = format!("blob {}\0", content.len());
        let mut store = Vec::new();
        store.extend_from_slice(header.as_bytes());
        store.extend_from_slice(&content);

        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&store)?;
        let compressed = encoder.finish()?;

        use sha1::{Digest, Sha1};
        let mut hasher = Sha1::new();
        hasher.update(&store);
        let hash = hasher.finalize();
        let hash_str = format!("{:x}", hash);

        Ok((hash_str, compressed))
    }
}
