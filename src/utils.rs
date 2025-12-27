use crate::objects::{blob, tree};
use crate::shared::types::generic_object::GenericObject;
use crate::shared::types::hash_result::HashResult;
use crate::shared::types::object_type::ObjectType;
use crate::shared::types::write_result::WriteResult;
use anyhow::{Context, bail};
use flate2::{Compression, bufread::ZlibDecoder, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{fs, io::Read, path::Path};

/// Decompresses zlib-compressed data using the DEFLATE algorithm.
/// Takes compressed bytes and returns the original uncompressed data
pub fn decompress(compressed: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

/// Computes the SHA-1 hash of the given data and returns it.
pub fn hash(data: &Vec<u8>) -> anyhow::Result<String> {
    let mut hasher = Sha1::new();
    hasher.update(&data);
    let object_hash = format!("{:x}", hasher.finalize());
    Ok(object_hash)
}

/// Compresses data using zlib compression with default compression level.
/// Returns the compressed bytes.
pub fn compress(data: &Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&data)?;
    let compressed_content = encoder.finish()?;
    Ok(compressed_content)
}

/// Reads a git object from `.flux/objects` given its hash.
///
/// Locates the object on disk, decompresses it, parses the header and validates the content size.  
/// Returns a `GenericObject` containing:
/// - `object_type`
/// - `size`
/// - `decompressed_content`
pub fn read_object(store_dir: &Path, object_hash: &str) -> anyhow::Result<GenericObject> {
    let (dir, file) = object_hash.split_at(2);
    let object_path = store_dir.join("objects").join(dir).join(file);

    let compressed_content = fs::read(object_path)?;
    let decompressed = decompress(compressed_content)?;

    let null_pos = decompressed
        .iter()
        .position(|&b| b == b'\0')
        .ok_or_else(|| anyhow::anyhow!("Invalid object: no null byte"))?;

    let header = String::from_utf8(decompressed[..null_pos].to_vec())?;
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
    let decompressed_content = decompressed[null_pos + 1..].to_vec();

    if decompressed_content.len() != size {
        bail!(
            "Size mismatch: expected {}, got {}",
            size,
            decompressed_content.len()
        );
    }

    Ok(GenericObject {
        object_type,
        size,
        decompressed_content,
    })
}

/// Writes a git object to the `.flux/objects` directory, given the object's `compressed` contents
pub fn store_object(store_dir: &Path, hash: &str, compressed_data: &[u8]) -> anyhow::Result<()> {
    let (dir, file) = hash.split_at(2);
    let object_dir = store_dir.join("objects").join(dir);
    let object_path = object_dir.join(file);

    fs::create_dir_all(&object_dir)?;

    let temp_path: std::path::PathBuf = object_path.with_extension("tmp");
    fs::write(&temp_path, compressed_data)?;
    fs::rename(temp_path, object_path)?;

    Ok(())
}

/// Writes either a `file` or a `dir` to the object storage inside `.flux/objects` given it's path
pub fn write_object(
    store_dir: &Path,
    work_tree: &Path,
    full_path: &Path,
) -> anyhow::Result<WriteResult> {
    let metadata = fs::metadata(&full_path).context("Failed to read file metadata")?;
    let mode: String;
    let result: HashResult;

    if metadata.is_file() {
        let perm = metadata.permissions().mode();
        if perm & 0o111 != 0 {
            mode = "100755".to_string();
        } else {
            mode = "100644".to_string();
        }
        let content = fs::read(&full_path)?;
        let blob = blob::hash_blob(content)?;
        store_object(store_dir, &blob.object_hash, &blob.compressed_content)?;
        result = blob;
    } else if metadata.is_dir() {
        mode = "40000".to_string();
        let builder = tree::TreeBuilder { work_tree, store_dir };
        result = builder.write_tree(&PathBuf::from(full_path))?;
    } else {
        bail!("Unsupported file type");
    }

    Ok(WriteResult {
        hash: result.object_hash,
        mode,
    })
}

///Gets the `hash` for a given `file` or `directory`
pub fn get_hash(store_dir: &Path, work_tree: &Path, full_path: &Path) -> anyhow::Result<String> {
    let metadata = fs::metadata(&full_path).context("Failed to read file metadata")?;

    let hash = if metadata.is_file() {
        let content = fs::read(&full_path)?;
        let res = blob::hash_blob(content)?;
        res.object_hash
    } else if metadata.is_dir() {
        let builder = tree::TreeBuilder { work_tree, store_dir };
        let res = builder.write_tree(&PathBuf::from(full_path))?;
        res.object_hash
    } else {
        bail!("Unsupported file type");
    };

    Ok(hash)
}
