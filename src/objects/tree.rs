use crate::objects::blob;
use crate::shared::types::object_type::ObjectType;
use crate::shared::types::{hash_result::HashResult, tree_entry::TreeEntry};
use crate::utils::{self, read_object};
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

pub struct TreeBuilder<'a> {
    pub work_tree: &'a Path,
    pub store_dir: &'a Path,
}

impl<'a> TreeBuilder<'a> {
    pub fn write_tree(&self, path: &Path) -> Result<HashResult> {
        let entries = self.collect_entries(path)?;
        let tree_content = build_tree_content(entries);
        let result = hash_tree(tree_content)?;

        utils::store_object(
            self.store_dir,
            &result.object_hash,
            &result.compressed_content,
        )?;

        Ok(result)
    }

    fn collect_entries(&self, path: &Path) -> Result<Vec<TreeEntry>> {
        let mut entries = Vec::new();

        for dir_entry in
            fs::read_dir(path).with_context(|| format!("Failed to read directory {:?}", path))?
        {
            let dir_entry = dir_entry?;
            let entry_path = dir_entry.path();

            if entry_path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                continue;
            }

            let name = dir_entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("Invalid filename"))?;

            let metadata = fs::metadata(&entry_path)?;

            if metadata.is_file() {
                let content = fs::read(&entry_path)?;
                let blob = blob::hash_blob(content)?;

                utils::store_object(self.store_dir, &blob.object_hash, &blob.compressed_content)?;

                entries.push(TreeEntry {
                    mode: "100644".into(),
                    entry_type: "blob".into(),
                    hash: blob.object_hash,
                    name,
                });
            } else if metadata.is_dir() {
                let subtree = self.write_tree(&entry_path)?;

                entries.push(TreeEntry {
                    mode: "040000".into(),
                    entry_type: "tree".into(),
                    hash: subtree.object_hash,
                    name,
                });
            }
        }

        Ok(entries)
    }
}

pub fn hash_tree(tree_content: Vec<u8>) -> Result<HashResult> {
    let header = format!("tree {}\0", tree_content.len());
    let mut store = Vec::new();

    store.extend_from_slice(header.as_bytes());
    store.extend_from_slice(&tree_content);

    let object_hash = utils::hash(&store)?;
    let compressed_content = utils::compress(&store)?;

    Ok(HashResult {
        object_hash,
        compressed_content,
    })
}

pub fn build_tree_content(mut entries: Vec<TreeEntry>) -> Vec<u8> {
    entries.sort_by(|a, b| {
        let a_name = if a.mode == "040000" {
            format!("{}/", a.name)
        } else {
            a.name.clone()
        };

        let b_name = if b.mode == "040000" {
            format!("{}/", b.name)
        } else {
            b.name.clone()
        };

        a_name.cmp(&b_name)
    });

    let mut tree_content = Vec::new();

    for entry in entries {
        let hash_bytes = hex::decode(&entry.hash).expect("Invalid object hash");
        let entry_header = format!("{} {}\0", entry.mode, entry.name);

        tree_content.extend_from_slice(entry_header.as_bytes());
        tree_content.extend_from_slice(&hash_bytes);
    }

    tree_content
}

pub fn ls_tree(store_path: &Path, tree_hash: &str) -> anyhow::Result<String> {
    let entries = parse_tree(store_path, tree_hash)?;
    let mut result = String::new();
    
    for entry in entries {
        let object_type = if entry.mode.starts_with("040") {
            "tree"
        } else {
            "blob"
        };
        
        result.push_str(&format!(
            "{} {} {} {}\n",
            entry.mode, object_type, entry.hash, entry.name
        ));
    }
    
    Ok(result)
}

pub fn parse_tree(store_path: &Path, tree_hash: &str) -> Result<Vec<TreeEntry>> {
    let object = read_object(&store_path, tree_hash)?;
    
    match object.object_type {
        ObjectType::Tree => {},
        _ => bail!("Unsupported object type")
    };
    
    let mut entries = Vec::new();
    let mut pos = 0;
    let content = object.decompressed_content;
    
    while pos < content.len() {
        let space_pos = content[pos..]
            .iter()
            .position(|&b| b == b' ')
            .context("Invalid tree format")?;
        
        let mode = String::from_utf8_lossy(&content[pos..pos + space_pos]).to_string();
        pos += space_pos + 1;
        
        let null_pos = content[pos..]
            .iter()
            .position(|&b| b == 0)
            .context("Invalid tree format")?;
        
        let name = String::from_utf8_lossy(&content[pos..pos + null_pos]).to_string();
        pos += null_pos + 1;
        
        let hash_bytes = &content[pos..pos + 20];
        let hash = hash_bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        pos += 20;
        
        entries.push(TreeEntry { mode, name, hash, entry_type: "tree".to_string() });
    }

    Ok(entries)
}