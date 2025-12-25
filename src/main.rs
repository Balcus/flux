use clap::Parser;
use flux::{
    cli::{Cli, Commands},
    commands,
    repo::repository::Repository,
};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            Repository::init(path, false)?;
        }
        Commands::Set { key, value } => {
            commands::set(key, value)?;
        }
        Commands::CatFile { object_hash, .. } => {
            commands::cat_file(object_hash)?;
        }
        Commands::HashObject { path, write } => {
            commands::hash_object(path, write)?;
        }
        Commands::LsTree { tree_hash, .. } => {
            commands::ls_tree(tree_hash)?;
        }
        Commands::CommitTree { tree_hash, message } => {
            commands::commit_tree(tree_hash, message)?;
        }
        Commands::Add { .. } => {}
    }
    Ok(())
}
