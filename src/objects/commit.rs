use crate::utils;
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

pub fn show_commit(git_path: &Path, commit_hash: String) -> anyhow::Result<()> {
    let commit = utils::read_object(git_path, &commit_hash)?;
    println!("{}", String::from_utf8(commit.decompressed_content)?);
    Ok(())
}
