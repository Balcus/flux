use crate::repo::repository::Repository;

pub fn set(key: String, value: String) -> anyhow::Result<()> {
    let mut repository = Repository::open(None)?;
    repository.set(key, value)?;
    Ok(())
}

pub fn cat_file(hash: String) -> anyhow::Result<()> {
    let repository = Repository::open(None)?;
    repository.cat_file(&hash)?;
    Ok(())
}

pub fn hash_object(path: String, write: bool) -> anyhow::Result<String> {
    let repository = Repository::open(None)?;
    let hash = repository.hash_object(path, write)?;
    println!("{hash}");
    Ok(hash)
}

pub fn ls_tree(hash: String) -> anyhow::Result<()> {
    let repository = Repository::open(None)?;
    repository.ls_tree(&hash)?;
    Ok(())
}

pub fn commit_tree(
    tree_hash: String,
    message: String,
    parent_hash: Option<String>,
) -> anyhow::Result<()> {
    let repository = Repository::open(None)?;
    let hash = repository.commit_tree(tree_hash, message, parent_hash)?;
    println!("{hash}");
    Ok(())
}

pub fn add(path: String) -> anyhow::Result<()> {
    let mut repository = Repository::open(None)?;
    repository.add(&path)?;
    println!("Added {path} to index");
    Ok(())
}

pub fn remove(path: String) -> anyhow::Result<()> {
    let mut repository = Repository::open(None)?;
    repository.delete(&path)?;
    println!("Deleted {path} from index");
    Ok(())
}

pub fn write_index() -> anyhow::Result<()> {
    let repository = Repository::open(None)?;
    let hash = repository.tree_from_index()?;
    println!("{hash}");
    Ok(())
}

pub fn commit(message: String) -> anyhow::Result<String> {
    let mut repository = Repository::open(None)?;
    let hash = repository.commit(message)?;
    println!("{hash}");
    Ok(hash)
}

pub fn log() -> anyhow::Result<()> {
    let repository = Repository::open(None)?;
    repository.log(None)?;
    Ok(())
}

pub fn split(name: String) -> anyhow::Result<()> {
    let repository = Repository::open(None)?;
    repository.new_branch(name)?;
    Ok(())
}

pub fn show_branches() -> anyhow::Result<()> {
    let repository = Repository::open(None)?;
    let output = repository.show_branches()?;
    println!("{output}");
    Ok(())
}

pub fn create_branch(name: String) -> anyhow::Result<()> {
    let repository = Repository::open(None)?;
    repository.new_branch(name)?;
    Ok(())
}

pub fn delete_branch(_name: String) -> anyhow::Result<()> {
    todo!()
}

pub fn switch_branch(name: String, force: bool) -> anyhow::Result<()> {
    let mut repository = Repository::open(None)?;
    repository.switch_branch(name, force)?;
    Ok(())
}
