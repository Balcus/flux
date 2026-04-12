use crate::{
    commands::{command::Command, hash_object::HashObject},
    database::database::Database,
    diff::diff_impl::Mayers,
    dircache::index::Index,
    internals::repository::Repository,
    status::{change_type::ChangeType, status_impl::Status},
};
use anyhow::Result;
use std::fs;

const NULL_ID: &str = "0000000";
const NULL_PATH: &str = "dev/null";

//TODO: error handling for commands !!
// some standardized way to deal with the short form of object ids
// STOP IGNORING FILE MODES
#[derive(Debug)]
struct Target {
    path: String,
    id: String,
    mode: Option<String>,
    data: Option<String>,
}

impl Target {
    fn new(path: String, id: String, mode: Option<String>, data: Option<String>) -> Self {
        Self {
            path,
            id,
            mode,
            data,
        }
    }

    fn diff_path(&self) -> &str {
        match self.mode {
            Some(_) => &self.path,
            None => NULL_PATH,
        }
    }
}

pub struct DiffCommand<'a> {
    repo: &'a mut Repository,
    status: Status,
    staged: bool,
}

impl<'a> DiffCommand<'a> {
    pub fn new(repo: &'a mut Repository, staged: bool) -> Result<Self> {
        let status = Status::new(repo)?;
        Ok(Self {
            repo,
            status,
            staged,
        })
    }

    fn print_diff(&self, a: &Target, b: &Target) {
        if a.id == b.id && a.mode == b.mode {
            return;
        }
        let a_path = format!("a/{}", a.path);
        let b_path = format!("b/{}", b.path);
        println!("diffing {} {}", a_path, b_path);
        self.print_diff_mode(a, b);
        self.print_diff_content(a, b);
        println!();
    }

    fn print_diff_mode(&self, a: &Target, b: &Target) {
        match (&a.mode, &b.mode) {
            (_, None) => println!("deleted file mode {}", a.mode.as_ref().unwrap()),
            (None, _) => println!("added file mode {}", b.mode.as_ref().unwrap()),
            (Some(a_mode), Some(b_mode)) if a_mode != b_mode => {
                println!("old mode {}", a_mode);
                println!("new mode {}", b_mode);
            }
            _ => {}
        }
    }

    fn print_diff_content(&self, a: &Target, b: &Target) {
        if a.id == b.id {
            return;
        }
        let mut id_range = format!("index {}..{}", a.id, b.id);
        if a.mode == b.mode
            && let Some(mode) = &a.mode
        {
            id_range.push_str(&format!(" {}", mode));
        }
        println!("{}", id_range);
        println!("--- {}", a.diff_path());
        println!("+++ {}", b.diff_path());

        let a_data = a.data.as_deref().unwrap_or("");
        let b_data = b.data.as_deref().unwrap_or("");

        let hunks = Mayers::diff_hunks(a_data, b_data);
        hunks.iter().for_each(|hunk| println!("{hunk}"));
    }

    fn from_index(&self, path: &str) -> Target {
        let mut index = Index::new(self.repo.flux_dir.join("index"));
        index.load().unwrap();
        let entry = index.entries.get(&(path.to_string(), 0)).unwrap();
        let hash = hex::encode(entry.id);
        let db = Database::open(self.repo.flux_dir.clone());
        let short_id = db.short_id(&hash).to_string();
        let blob = db.read_object(&hash).unwrap();
        let data = String::from_utf8(blob.content()).ok();
        let mode = Some("100644".to_string());
        Target::new(path.to_string(), short_id, mode, data)
    }

    fn from_workspace(&self, path: &str) -> Target {
        let db = Database::open(self.repo.flux_dir.clone());
        let id = db.short_id(&HashObject::new(path, false).hash(None).unwrap());
        let data = fs::read_to_string(path).ok();
        let mode = Some("100644".to_string());
        Target::new(path.to_string(), id, mode, data)
    }

    fn from_nothing(&self) -> Target {
        Target::new(NULL_PATH.to_string(), NULL_ID.into(), None, None)
    }

    fn from_head(&self, path: &str) -> Target {
        let db = Database::open(self.repo.flux_dir.clone());
        let id = self.status.head_tree.get(path).unwrap();
        let short_id = db.short_id(id);
        let blob = db.read_object(id).unwrap();
        let data = String::from_utf8(blob.content()).ok();
        let mode = Some("100644".to_string());
        Target::new(path.to_string(), short_id, mode, data)
    }

    fn diff_head_index(&self) {
        for (path, state) in &self.status.index_changes {
            let (a, b) = match state {
                ChangeType::Modified => (self.from_head(path), self.from_index(path)),
                ChangeType::Deleted => (self.from_head(path), self.from_nothing()),
                ChangeType::Added => (self.from_nothing(), self.from_index(path)),
            };
            self.print_diff(&a, &b);
        }
    }

    fn diff_index_workspace(&self) {
        for (path, state) in &self.status.workspace_changes {
            let (a, b) = match state {
                ChangeType::Modified => (self.from_index(path), self.from_workspace(path)),
                ChangeType::Deleted => (self.from_index(path), self.from_nothing()),
                ChangeType::Added => (self.from_nothing(), self.from_workspace(path)),
            };
            self.print_diff(&a, &b);
        }
    }
}

impl<'a> Command for DiffCommand<'a> {
    fn run(&mut self) -> Result<()> {
        match self.staged {
            false => self.diff_index_workspace(),
            true => self.diff_head_index(),
        }
        Ok(())
    }
}
