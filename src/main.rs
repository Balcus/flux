use clap::Parser;
use flux::{cli::{Cli, Commands}, commands, repo::repository::Repository};

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => { let _ = Repository::init(path); },
        Commands::Set { key, value } => { let _ = commands::set(key, value); },
        Commands::CatFile { object_hash, .. } => { let _ = commands::cat_file(object_hash); },
        Commands::HashObject { path, write } => { let _ = commands::hash_object(path, write); },
        Commands::LsTree { tree_hash, .. } => { let _ = commands::ls_tree(tree_hash); }
        Commands::Add { .. } => {}
    }
    Ok(())
}