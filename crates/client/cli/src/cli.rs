use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Path to the repository (defaults to current directory)
    #[arg(long = "repo-path", global = true)]
    pub repo_path: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum BranchCommands {
    /// Show all branches
    Show {},

    /// Create a new branch with the given name
    New {
        #[arg(value_name = "branch-name")]
        name: String,
    },

    /// Delete a specified branch
    Delete {
        #[arg(value_name = "branch-name")]
        name: String,
    },

    /// Switch to another branch
    ///
    /// By default, the switch will fail if there are uncommitted changes.
    /// Use --force to override this behavior.
    Switch {
        #[arg(value_name = "branch-name")]
        name: String,

        #[arg(short = 'f', long = "force", default_value_t = false)]
        /// Force the branch switch even if there are uncommitted changes
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new Flux repository
    ///
    /// Creates the repository structure in the specified directory.
    /// If no path is provided, the current directory is used.
    Init {
        /// Target directory for the repository
        path: Option<String>,

        #[arg(short = 'f', long = "force", default_value_t = false)]
        // If enabled will delete the previous .flux directory and will create a new one
        force: bool,
    },

    /// Set a configuration value
    Set {
        key: String,
        value: String,
    },

    /// Display the contents of a repository object
    ///
    /// - Blobs: prints the raw file contents
    /// - Trees: lists entries with mode, type, name, and hash
    /// - Commits: shows the tree hash and commit metadata
    CatFile {
        /// Pretty-print the object contents
        #[arg(short = 'p')]
        pretty_print: bool,

        #[arg(value_name = "object-hash")]
        object_hash: String,
    },

    /// Compute the object hash for a file or directory
    ///
    /// By default, this only prints the hash.
    /// Use -w to write the object into the object store.
    HashObject {
        /// Write the object to the object database
        #[arg(short = 'w')]
        write: bool,

        path: String,
    },

    /// Add a file or directory to the staging area
    Add {
        path: String,
    },

    /// Remove a file or directory from the staging area
    Delete {
        path: String,
    },

    /// Create a commit object from a tree
    ///
    /// This command manually constructs a commit using a tree hash.
    CommitTree {
        tree_hash: String,

        /// Commit message
        #[arg(short = 'm', long = "message")]
        message: String,

        /// Parent commit hash (can be ommited)
        #[arg(short = 'p', long = "parent")]
        parent_hash: Option<String>,
    },

    /// Create a new commit from the current index
    Commit {
        /// Commit message
        #[arg(short = 'm', long = "message")]
        message: String,
    },

    /// Show the commit history
    Log {},

    /// Manage branches
    Branch {
        #[command(subcommand)]
        subcommand: BranchCommands,
    },

    Push {
        url: Option<String>,
    },

    RestoreFs {},
    Clone {
        url: String,
        path: Option<String>,
    },
}
