use crate::shared::types::generic_object::GenericObject;
use crate::shared::types::object_type::ObjectType;
use anyhow::bail;
use flate2::{Compression, bufread::ZlibDecoder, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use std::io::Write;
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

/// Reads a git object from `.git/objects` given its hash.
///
/// Locates the object on disk, decompresses it, parses the header and validates the content size.  
/// Returns a `GenericObject` containing:
/// - `object_type`
/// - `size`
/// - `decompressed_content`
pub fn read_object(git_path: &Path, object_hash: &str) -> anyhow::Result<GenericObject> {
    let (dir, file) = object_hash.split_at(2);
    let object_path = git_path.join("objects").join(dir).join(file);

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

/// Writes a git object to the `.git/objects` directory, given the object's `compressed` contents
pub fn write_object(git_dir: &Path, hash: &str, compressed_data: &[u8]) -> anyhow::Result<()> {
    let (dir, file) = hash.split_at(2);
    let object_dir = git_dir.join("objects").join(dir);
    let object_path = object_dir.join(file);

    fs::create_dir_all(&object_dir)?;

    let temp_path: std::path::PathBuf = object_path.with_extension("tmp");
    fs::write(&temp_path, compressed_data)?;
    fs::rename(temp_path, object_path)?;

    Ok(())
}
