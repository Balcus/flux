use crate::internals::repository::Repository;

pub fn set(repo_path: Option<String>, key: String, value: String) -> anyhow::Result<()> {
    let mut repository = Repository::open(repo_path)?;
    repository.set(key, value)?;
    Ok(())
}

pub fn cat_file(repo_path: Option<String>, hash: String) -> anyhow::Result<()> {
    let repository = Repository::open(repo_path)?;
    repository.cat(&hash)?;
    Ok(())
}

pub fn hash_object(repo_path: Option<String>, path: String, write: bool) -> anyhow::Result<String> {
    let repository = Repository::open(repo_path)?;
    let hash = repository.hash_object(path, write)?;
    println!("{hash}");
    Ok(hash)
}

pub fn commit_tree(
    repo_path: Option<String>,
    tree_hash: String,
    message: String,
    parent_hash: Option<String>,
) -> anyhow::Result<()> {
    let repository = Repository::open(repo_path)?;
    repository.commit_tree(tree_hash, message, parent_hash)?;
    Ok(())
}

pub fn add(repo_path: Option<String>, path: String) -> anyhow::Result<()> {
    let mut repository = Repository::open(repo_path)?;
    repository.add(&path)?;
    println!("Added {path} to index");
    Ok(())
}

pub fn remove(repo_path: Option<String>, path: String) -> anyhow::Result<()> {
    let mut repository = Repository::open(repo_path)?;
    repository.delete(&path)?;
    println!("Deleted {path} from index");
    Ok(())
}

pub fn commit(repo_path: Option<String>, message: String) -> anyhow::Result<String> {
    let mut repository = Repository::open(repo_path)?;
    let hash = repository.commit(message)?;
    println!("{hash}");
    Ok(hash)
}

pub fn log(repo_path: Option<String>) -> anyhow::Result<()> {
    let repository = Repository::open(repo_path)?;
    repository.log(None)?;
    Ok(())
}

pub fn show_branches(repo_path: Option<String>) -> anyhow::Result<()> {
    let repository = Repository::open(repo_path)?;
    let output = repository.show_branches()?;
    println!("{output}");
    Ok(())
}

pub fn create_branch(repo_path: Option<String>, name: String) -> anyhow::Result<()> {
    let mut repository = Repository::open(repo_path)?;
    repository.new_branch(&name);
    Ok(())
}

pub fn delete_branch(_repo_path: Option<String>, _name: String) -> anyhow::Result<()> {
    todo!()
}

pub fn switch_branch(repo_path: Option<String>, name: String, force: bool) -> anyhow::Result<()> {
    let mut repository = Repository::open(repo_path)?;
    repository.switch_branch(&name, force);
    Ok(())
}
