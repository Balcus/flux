use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init {
        path: Option<String>,
    },
    Set {
        key: String,
        value: String,
    },
    CatFile {
        #[arg(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    HashObject {
        #[arg(short = 'w')]
        write: bool,

        path: String,
    },
    LsTree {
        #[arg(long = "name-only")]
        name_only: bool,

        tree_hash: String,
    },
    Add {
        path: String,
    },
    Delete {
        path: String,
    },
    CommitTree {
        tree_hash: String,

        #[arg(short = 'm', long = "message")]
        message: String,

        #[arg(short = 'p', long = "parent")]
        parent_hash: Option<String>,
    },
    WriteIndex {},
    Commit {
        #[arg(short = 'm', long = "message")]
        message: String,
    },
}
