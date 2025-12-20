use anyhow::Ok;
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

    pub fn ls_tree(_size: usize, content: Vec<u8>) -> anyhow::Result<String> {
        let mut result: String = String::new();
        let mut i = 0;

        while i < content.len() {
            let mode_end = content[i..].iter().position(|&b| b == b' ').unwrap();
            let mode = std::str::from_utf8(&content[i..i + mode_end])?;
            i += mode_end + 1;

            let name_end = content[i..].iter().position(|&b| b == b'\0').unwrap();
            let name = std::str::from_utf8(&content[i..i + name_end])?;
            i += name_end + 1;

            let hash = hex::encode(&content[i..i + 20]);
            i += 20;

            result.push_str(&format!("{mode} {name} {hash}\n"));
        }

        Ok(result)
    }
}
