use crate::error;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Refs {
    pub refs_path: PathBuf,
    pub branches: HashMap<String, String>,
    pub head_path: PathBuf,
}

pub type Result<T> = std::result::Result<T, error::RefsError>;

impl Refs {
    fn parse_head_ref(head_contents: &str) -> Result<String> {
        let s = head_contents.trim();

        let r = s
            .strip_prefix("ref: ")
            .ok_or_else(|| error::RefsError::InvalidHead {
                head: s.to_string(),
            })?;

        if !r.starts_with("refs/heads/") {
            return Err(error::RefsError::InvalidHead {
                head: r.to_string(),
            })?;
        }

        Ok(r.to_string())
    }

    pub fn new(flux_dir: &Path) -> Result<Self> {
        let refs_path = flux_dir.join("refs");
        let head_path = flux_dir.join("HEAD");
        let heads_path = refs_path.join("heads");
        let main_path = heads_path.join("main");

        fs::create_dir_all(&heads_path)
            .map_err(|e| error::IoError::create_error(&heads_path, e))?;

        File::create(&main_path).map_err(|e| error::IoError::create_error(&main_path, e))?;
        fs::write(&main_path, "").map_err(|e| error::IoError::write_error(&main_path, e))?;

        fs::write(&head_path, "ref: refs/heads/main\n")
            .map_err(|e| error::IoError::write_error(&head_path, e))?;

        let mut branches = HashMap::new();
        branches.insert("main".to_string(), "".to_string());

        Ok(Self {
            refs_path,
            branches,
            head_path,
        })
    }

    pub fn load(flux_dir: &Path) -> Result<Self> {
        let refs_path = flux_dir.join("refs");
        let heads_path = refs_path.join("heads");

        if !refs_path.is_dir() {
            return Err(error::IoError::missing_error(&refs_path).into());
        }
        if !heads_path.is_dir() {
            return Err(error::IoError::missing_error(&heads_path).into());
        }

        let heads =
            fs::read_dir(&heads_path).map_err(|e| error::IoError::read_error(&heads_path, e))?;

        let mut map: HashMap<String, String> = HashMap::new();
        for entry_res in heads {
            let entry = entry_res.map_err(|e| error::IoError::read_error(&heads_path, e))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            let head =
                fs::read_to_string(&path).map_err(|e| error::IoError::read_error(&path, e))?;

            map.insert(name, head.trim().to_string());
        }

        Ok(Self {
            refs_path,
            branches: map,
            head_path: flux_dir.join("HEAD"),
        })
    }

    pub fn head_ref(&self) -> Result<String> {
        let raw = fs::read_to_string(&self.head_path)
            .map_err(|e| error::IoError::read_error(&self.head_path, e))?;
        Self::parse_head_ref(&raw)
    }

    pub fn current_branch(&self) -> Result<String> {
        let head_ref = self.head_ref()?;

        let name =
            head_ref
                .strip_prefix("refs/heads/")
                .ok_or_else(|| error::RefsError::InvalidHead {
                    head: head_ref.clone(),
                })?;

        Ok(name.to_string())
    }

    pub fn head_ref_path(&self) -> Result<PathBuf> {
        let head_ref = self.head_ref()?;
        let rel = head_ref
            .strip_prefix("refs/")
            .ok_or_else(|| error::RefsError::InvalidHead {
                head: head_ref.clone(),
            })?;
        Ok(self.refs_path.join(rel))
    }

    pub fn head_commit(&self) -> Result<String> {
        let branch_path = self.head_ref_path()?;
        let last_commit = fs::read_to_string(&branch_path)
            .map_err(|e| error::IoError::read_error(&branch_path, e))?;
        Ok(last_commit.trim().to_string())
    }

    pub fn set_head(&self, branch: &str) -> Result<()> {
        fs::write(&self.head_path, format!("ref: refs/heads/{}\n", branch))
            .map_err(|e| error::IoError::write_error(&self.head_path, e))?;
        Ok(())
    }

    pub fn new_branch(&mut self, name: &str) -> Result<()> {
        let path = self.refs_path.join("heads").join(name);

        if path.exists() {
            return Err(error::RefsError::BranchAlreadyExists(name.to_string()));
        }

        let start_commit = self.head_commit()?;
        fs::write(&path, start_commit.as_bytes())
            .map_err(|e| error::IoError::write_error(&path, e))?;

        self.branches.insert(name.to_string(), start_commit);
        self.set_head(name)?;

        Ok(())
    }

    pub fn delete_branch(&mut self, name: &str) -> Result<()> {
        let current = self.current_branch()?;
        if name == current {
            return Err(error::RefsError::DeleteCurrentBranch(name.to_string()))?;
        }

        let path = self.refs_path.join("heads").join(name);
        if !path.is_file() {
            return Err(error::RefsError::MissingBranch(name.to_string()))?;
        }

        fs::remove_file(&path).map_err(|e| error::IoError::delete_error(&path, e))?;
        self.branches.remove(name);
        Ok(())
    }

    pub fn switch_branch(&mut self, to: &str) -> Result<()> {
        let path = self.refs_path.join("heads").join(to);
        if !path.is_file() {
            return Err(error::RefsError::MissingBranch(to.to_string()))?;
        }
        self.set_head(to)?;
        Ok(())
    }

    pub fn update_head(&mut self, commit_hash: &str) -> Result<()> {
        let path = self.head_ref_path()?;
        fs::write(&path, commit_hash.as_bytes())
            .map_err(|e| error::IoError::write_error(&path, e))?;

        let branch = self.current_branch()?;
        self.branches.insert(branch, commit_hash.to_string());

        Ok(())
    }

    pub fn branch_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.branches.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn format_branches(&self) -> Result<String> {
        let current = self.current_branch()?;
        let mut out = String::new();

        for name in self.branch_names() {
            if name == current {
                out.push_str("(*) ");
            } else {
                out.push_str("  ");
            }
            out.push_str(&name);
            out.push('\n');
        }

        Ok(out)
    }

    pub fn list_branches(&self) -> Result<Vec<String>> {
        let current = self.current_branch()?;
        let mut res = Vec::new();

        for name in self.branch_names() {
            if name == current {
                res.push(format!("(*) {}", name));
            } else {
                res.push(format!("    {}", name));
            }
        }

        Ok(res)
    }
}
