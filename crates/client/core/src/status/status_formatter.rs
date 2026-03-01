use std::collections::HashMap;

use crate::{status::status_impl::{ChangeType, Status}, utils::colors::{GREEN, RED, RESET, YELLOW}};

pub struct StatusFormatter<'a> {
    changes: &'a Status,
}

impl<'a> StatusFormatter<'a> {
    pub fn new(changes: &'a Status) -> Self {
        Self { changes }
    }

    pub fn print(&self) {
        if self.changes.is_clean() {
            println!("Nothing to commit, working tree clean");
            return;
        }

        if self.changes.has_staged_changes() {
            println!("Changes to be committed:\n");
            self.print_staged_changes();
        }

        if self.changes.has_unstaged_changes() {
            println!("\nChanges not staged for commit:");
            println!("  (use \"flux add <file>...\" to stage changes)\n");
            self.print_unstaged_changes();
        }

        if self.changes.has_untracked_files() {
            println!("\nUntracked files:");
            println!("  (use \"flux add <file>...\" to track)\n");
            for file in &self.changes.untracked {
                println!("  {}{}{}", RED, file, RESET);
            }
        }
    }

    fn print_staged_changes(&self) {
        self.print_change_group(
            &self.changes.index_changes,
            "new file",
            ChangeType::Added,
            GREEN,
        );
        self.print_change_group(
            &self.changes.index_changes,
            "modified",
            ChangeType::Modified,
            YELLOW,
        );
        self.print_change_group(
            &self.changes.index_changes,
            "deleted",
            ChangeType::Deleted,
            GREEN,
        );
    }

    fn print_unstaged_changes(&self) {
        self.print_change_group(
            &self.changes.workspace_changes,
            "modified",
            ChangeType::Modified,
            RED,
        );
        self.print_change_group(
            &self.changes.workspace_changes,
            "deleted",
            ChangeType::Deleted,
            RED,
        );
    }

    fn print_change_group(
        &self,
        changes: &HashMap<String, ChangeType>,
        label: &str,
        change_type: ChangeType,
        color: &str,
    ) {
        let mut files: Vec<_> = changes
            .iter()
            .filter(|(_, ct)| **ct == change_type)
            .map(|(path, _)| path.clone())
            .collect();

        if !files.is_empty() {
            files.sort();
            for file in files {
                println!("  {}{}: {}{}", color, label, file, RESET);
            }
        }
    }
}