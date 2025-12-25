use crate::repo::repository::Repository;

pub fn set(key: String, value: String) -> Result<(), anyhow::Error> {
    let mut repository = Repository::open(None)?;
    repository.set(key, value)?;
    Ok(())
}

pub fn cat_file(hash: String) -> Result<(), anyhow::Error> {
    let repository = Repository::open(None)?;
    repository.cat_file(hash)?;
    Ok(())
}

pub fn hash_object(path: String, write: bool) -> Result<(), anyhow::Error> {
    let repository = Repository::open(None)?;
    let hash = repository.hash_object(path, write)?;
    println!("{hash}");
    Ok(())
}

pub fn ls_tree(hash: String) -> anyhow::Result<()> {
    let repository = Repository::open(None)?;
    repository.ls_tree(hash)?;
    Ok(())
}

pub fn commit_tree(tree_hash: String, message: String) -> anyhow::Result<()> {
    let repository = Repository::open(None)?;
    let hash = repository.commit_tree(tree_hash, message)?;
    println!("{hash}");
    Ok(())
}
