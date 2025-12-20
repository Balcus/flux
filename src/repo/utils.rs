use anyhow::bail;
use flate2::{Compression, bufread::ZlibDecoder, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use std::io::Write;
use std::{fs, io::Read, path::Path};

pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

pub struct GenericObject {
    pub object_type: ObjectType,
    pub size: usize,
    pub content: Vec<u8>,
}

pub struct HashResult {
    pub object_hash: String,
    pub compressed_content: Vec<u8>,
}

pub fn read_object(git_path: &Path, object_hash: &str) -> anyhow::Result<GenericObject> {
    let (dir, file) = object_hash.split_at(2);
    let object_path = git_path.join("objects").join(dir).join(file);

    let compressed_content = fs::read(object_path)?;
    let mut decoder = ZlibDecoder::new(&compressed_content[..]);

    let mut data = Vec::new();
    decoder.read_to_end(&mut data)?;

    let null_pos = data
        .iter()
        .position(|&b| b == b'\0')
        .ok_or_else(|| anyhow::anyhow!("Invalid object: no null byte"))?;

    let header = String::from_utf8(data[..null_pos].to_vec())?;
    let parts: Vec<&str> = header.split(' ').collect();

    if parts.len() != 2 {
        bail!("Invalid object header");
    }

    let object_type = match parts[0] {
        "blob" => ObjectType::Blob,
        "tree" => ObjectType::Tree,
        "commit" => ObjectType::Commit,
        _ => bail!("Unknown object type: {}", parts[0]),
    };

    let size: usize = parts[1].parse()?;
    let content = data[null_pos + 1..].to_vec();

    if content.len() != size {
        bail!("Size mismatch: expected {}, got {}", size, content.len());
    }

    Ok(GenericObject {
        object_type,
        size,
        content,
    })
}

pub fn hash_blob(content: Vec<u8>) -> anyhow::Result<HashResult> {
    let header = format!("blob {}\0", content.len());
    let mut store = Vec::new();
    store.extend_from_slice(header.as_bytes());
    store.extend_from_slice(&content);

    let mut hasher = Sha1::new();
    hasher.update(&store);
    let object_hash = format!("{:x}", hasher.finalize());

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&store)?;
    let compressed_content = encoder.finish()?;

    Ok(HashResult {
        object_hash,
        compressed_content,
    })
}

pub fn hash_tree(tree_content: Vec<u8>) -> anyhow::Result<HashResult> {
    let header = format!("tree {}\0", tree_content.len());
    let mut store = Vec::new();
    store.extend_from_slice(header.as_bytes());
    store.extend_from_slice(&tree_content);

    let mut hasher = Sha1::new();
    hasher.update(&store);
    let object_hash = format!("{:x}", hasher.finalize());

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&store)?;
    let compressed_content = encoder.finish()?;

    Ok(HashResult {
        object_hash,
        compressed_content,
    })
}

pub fn write_object(git_dir: &Path, hash: &str, compressed_data: &[u8]) -> anyhow::Result<()> {
    let (dir, file) = hash.split_at(2);
    let object_dir = git_dir.join("objects").join(dir);
    let object_path = object_dir.join(file);

    fs::create_dir_all(&object_dir)?;

    let temp_path = object_path.with_extension("tmp");
    fs::write(&temp_path, compressed_data)?;
    fs::rename(temp_path, object_path)?;

    Ok(())
}

pub struct TreeEntry {
    pub mode: String,
    pub hash: String,
    pub name: String,
    pub entry_type: String,
}

pub fn build_tree_content(mut entries: Vec<TreeEntry>) -> Vec<u8> {
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    let mut tree_content = Vec::new();

    for entry in entries {
        let hash_bytes = hex::decode(&entry.hash).expect("Invalid hash");
        let entry_str = format!("{} {} {}\0", entry.mode, entry.entry_type, entry.name);
        tree_content.extend_from_slice(entry_str.as_bytes());
        tree_content.extend_from_slice(&hash_bytes);
    }

    tree_content
}
