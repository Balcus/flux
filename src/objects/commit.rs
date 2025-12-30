use crate::{shared::{self, types::object_type::ObjectType}, utils};
use anyhow::bail;
use chrono::Local;
use std::path::Path;

pub fn commit_tree(
    store_dir: &Path,
    user_name: String,
    user_email: String,
    tree_hash: String,
    parent_hash: Option<String>,
    message: String,
) -> anyhow::Result<String> {
    let now = Local::now();

    let parent_line = match parent_hash {
        Some(h) => format!("parent {}\n", h),
        None => String::new(),
    };

    let commit_content = format!(
        "tree {}\n{}author {} <{}> {} {}\ncommitter {} <{}> {} {}\n\n{}",
        tree_hash,
        parent_line,
        user_name,
        user_email,
        now.timestamp(),
        now.format("%z"),
        user_name,
        user_email,
        now.timestamp(),
        now.format("%z"),
        message
    );

    let size = commit_content.len();
    let content = format!("commit {}\0{}", size, commit_content);
    let store: Vec<u8> = content.as_bytes().to_vec();

    let object_hash = utils::hash(&store)?;
    let compressed_content = utils::compress(&store)?;
    utils::store_object(store_dir, &object_hash, &compressed_content)?;

    Ok(object_hash)
}

pub fn show_commit(store_dir: &Path, commit_hash: &str) -> anyhow::Result<()> {
    let commit = utils::read_object(store_dir, &commit_hash)?;
    println!("{}\n\n", String::from_utf8(commit.decompressed_content)?);
    Ok(())
}

pub fn get_parent_hash(store_dir: &Path, commit_hash: String) -> anyhow::Result<Option<String>> {
    let commit = utils::read_object(store_dir, &commit_hash)?;

    match commit.object_type {
        ObjectType::Commit => {},
        _ => bail!("Parent of a commit must be itself a commit")
    };

    let content = String::from_utf8(commit.decompressed_content)?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("parent ") {
            return Ok(Some(rest.trim().to_string()));
        }

        if line.is_empty() {
            break;
        }
    }

    Ok(None)
}

pub fn get_tree_hash(commit_obj: shared::types::generic_object::GenericObject) -> anyhow::Result<Option<String>> {
    if commit_obj.object_type != ObjectType::Commit {
        bail!("Expected commit object");
    }

    let content = String::from_utf8(commit_obj.decompressed_content)?;

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("tree ") {
            return Ok(Some(rest.trim().to_string()));
        }

        if line.is_empty() {
            break;
        }
    }

    Ok(None)
}