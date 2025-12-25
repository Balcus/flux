use crate::{shared::types::hash_result::HashResult, utils};

pub fn hash_blob(content: Vec<u8>) -> anyhow::Result<HashResult> {
    let header = format!("blob {}\0", content.len());
    let mut store = Vec::new();
    store.extend_from_slice(header.as_bytes());
    store.extend_from_slice(&content);

    let object_hash = utils::hash(&store)?;
    let compressed_content = utils::compress(&store)?;

    Ok(HashResult {
        object_hash,
        compressed_content,
    })
}
