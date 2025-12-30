use clap::Parser;
use flux::{
    cli::{BranchCommands, Cli, Commands},
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
        Commands::CommitTree {
            tree_hash,
            message,
            parent_hash,
        } => {
            commands::commit_tree(tree_hash, message, parent_hash)?;
        }
        Commands::Add { path } => commands::add(path)?,
        Commands::Delete { path } => commands::remove(path)?,
        Commands::WriteIndex {} => commands::write_index()?,
        Commands::Commit { message } => {
            commands::commit(message)?;
        }
        Commands::Log {} => commands::log()?,
        Commands::Branch { subcommand } => match subcommand {
            BranchCommands::Show {} => commands::show_branches()?,
            BranchCommands::New { name } => commands::create_branch(name)?,
            BranchCommands::Delete { name } => commands::delete_branch(name)?,
            BranchCommands::Switch { name, force } => {
                commands::switch_branch(name, force)?
            }
        },
    }
    Ok(())
}
