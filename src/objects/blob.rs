use std::{fs, path::PathBuf, io::Write};
use anyhow::Context;
use crate::traits::GitObject;

pub struct Blob {}

impl GitObject for Blob {
    fn cat_file(work_tree: PathBuf, hash: String) -> Result<String, anyhow::Error> {
        let (object_type, _, content) = Self::read_object(work_tree, hash)?;

        if object_type != "blob" {
            anyhow::bail!("Object is not a blob");
        }

        let content_str = String::from_utf8(content)?;

        Ok(content_str)
    }

    fn hash_object(work_tree: PathBuf, path: String, write: bool) -> Result<String, anyhow::Error> {
        let path = work_tree.join(path);
        let content = fs::read(&path).with_context(|| format!("Could not read file {:?}", path))?;

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

        if write {
            let (dir, file) = hash_str.split_at(2);
            let object_path = work_tree.join(format!(".git/objects/{}/{}", dir, file));
            fs::create_dir_all(work_tree.join(format!(".git/objects/{}", dir)))?;
            fs::write(object_path, &compressed)?;
        }

        Ok(hash_str)
    }
}
