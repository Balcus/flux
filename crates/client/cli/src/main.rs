use crate::cli::{BranchCommands, Cli, Commands};
use clap::Parser;
use flux_core::internals::repository::Repository;

pub mod cli;
pub mod commands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let repo_path = cli.repo_path.clone();

    match cli.command {
        Commands::Init { path, force } => {
            Repository::init(path, force)?;
        }
        Commands::Set { key, value } => {
            commands::set(repo_path, key, value)?;
        }
        Commands::CatFile { object_hash, .. } => {
            commands::cat_file(repo_path, object_hash)?;
        }
        Commands::HashObject { path, write } => {
            commands::hash_object(repo_path, path, write)?;
        }
        Commands::CommitTree {
            tree_hash,
            message,
            parent_hash,
        } => {
            commands::commit_tree(repo_path, tree_hash, message, parent_hash)?;
        }
        Commands::Add { path } => {
            commands::add(repo_path, path)?;
        }
        Commands::Delete { path } => {
            commands::remove(repo_path, path)?;
        }
        Commands::Commit { message } => {
            commands::commit(repo_path, message)?;
        }
        Commands::Log {} => {
            commands::log(repo_path)?;
        }
        Commands::Branch { subcommand } => match subcommand {
            BranchCommands::Show {} => {
                commands::show_branches(repo_path)?;
            }
            BranchCommands::New { name } => {
                commands::create_branch(repo_path, name)?;
            }
            BranchCommands::Delete { name } => {
                commands::delete_branch(repo_path, name)?;
            }
            BranchCommands::Switch { name, force } => {
                commands::switch_branch(repo_path, name, force)?;
            }
        },
        Commands::Push { url } => {
            commands::push(repo_path, url).await?;
        }
        Commands::RestoreFs {} => {
            commands::restore_fs(repo_path)?;
        }
        Commands::Clone { url, path } => {
            commands::clone(url, path).await?;
        },
        Commands::Status {  } => {
            commands::status(repo_path)?;
        },
        Commands::Auth { url } => {
            commands::auth(repo_path, url).await?;
        }
    }

    Ok(())
}
