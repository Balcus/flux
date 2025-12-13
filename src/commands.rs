use crate::{repo::repository::Repository, traits::GitObject};
use std::path::PathBuf;

pub fn set(key: String, value: String) -> Result<(), anyhow::Error> {
    let mut repository = Repository::open(None::<PathBuf>)?;
    repository.set(key, value)?;
    Ok(())
}

pub fn cat_file(hash: String) -> Result<(), anyhow::Error> {
    let repository = Repository::open(None::<PathBuf>)?;
    repository.cat_file(hash)?;
    Ok(())
}

pub fn hash_object(path: String, write: bool) -> Result<(), anyhow::Error> {
    let repository = Repository::open(None::<PathBuf>)?;
    let hash = crate::objects::blob::Blob::hash_object(repository.work_tree, path, write)?;
    println!("{}", hash);
    Ok(())
}