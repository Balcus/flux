use anyhow::{Context, bail};

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Refs {
    pub refs_path: PathBuf,
    pub branches: HashMap<String, String>,
    pub head_path: PathBuf,
}

// TODO: change the switch branch logic, currently a commit no longer clears the index (it shouldnt have either)
impl Refs {
    fn parse_head_ref(head_contents: &str) -> anyhow::Result<String> {
        let s = head_contents.trim();
        let r = s
            .strip_prefix("ref: ")
            .with_context(|| format!("Invalid head format: '{s}'."))?;

        if !r.starts_with("refs/heads/") {
            bail!("Invalid head format: '{r}'.");
        }

        Ok(r.to_string())
    }

    pub fn new(flux_dir: &Path) -> anyhow::Result<Self> {
        let refs_path = flux_dir.join("refs");
        let head_path = flux_dir.join("HEAD");
        let heads_path = refs_path.join("heads");
        let main_path = heads_path.join("main");

        fs::create_dir_all(&heads_path)
            .with_context(|| format!("Failed to create '{}'.", heads_path.display()))?;
        fs::write(&main_path, "")
            .with_context(|| format!("Failed to write '{}'.", main_path.display()))?;
        fs::write(&head_path, "ref: refs/heads/main\n")
            .with_context(|| format!("Failed to write '{}'.", head_path.display()))?;

        let mut branches = HashMap::new();
        branches.insert("main".to_string(), "".to_string());

        Ok(Self {
            refs_path,
            branches,
            head_path,
        })
    }

    pub fn load(flux_dir: &Path) -> anyhow::Result<Self> {
        let refs_path = flux_dir.join("refs");
        let heads_path = refs_path.join("heads");

        if !refs_path.is_dir() {
            bail!("Missing required path '{}'.", refs_path.display());
        }
        if !heads_path.is_dir() {
            bail!("Missing required path '{}'.", heads_path.display());
        }

        let heads = fs::read_dir(&heads_path)
            .with_context(|| format!("Failed to read '{}'.", heads_path.display()))?;

        let mut map: HashMap<String, String> = HashMap::new();
        for entry_res in heads {
            let entry =
                entry_res.with_context(|| format!("Failed to read '{}'.", heads_path.display()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            let head = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read '{}'.", path.display()))?;
            map.insert(name, head.trim().to_string());
        }

        Ok(Self {
            refs_path,
            branches: map,
            head_path: flux_dir.join("HEAD"),
        })
    }

    pub fn head_ref(&self) -> anyhow::Result<String> {
        let raw = fs::read_to_string(&self.head_path)
            .with_context(|| format!("Failed to read '{}'.", self.head_path.display()))?;
        Self::parse_head_ref(&raw)
    }

    pub fn current_branch(&self) -> anyhow::Result<String> {
        let head_ref = self.head_ref()?;
        let name = head_ref
            .strip_prefix("refs/heads/")
            .with_context(|| format!("Invalid head format: '{head_ref}'."))?;
        Ok(name.to_string())
    }

    pub fn head_ref_path(&self) -> anyhow::Result<PathBuf> {
        let head_ref = self.head_ref()?;
        let rel = head_ref
            .strip_prefix("refs/")
            .with_context(|| format!("Invalid head format: '{head_ref}'."))?;
        Ok(self.refs_path.join(rel))
    }

    pub fn head_commit(&self) -> anyhow::Result<String> {
        let branch_path = self.head_ref_path()?;
        let last_commit = fs::read_to_string(&branch_path)
            .with_context(|| format!("Failed to read '{}'.", branch_path.display()))?;
        Ok(last_commit.trim().to_string())
    }

    pub fn set_head(&self, branch: &str) -> anyhow::Result<()> {
        fs::write(&self.head_path, format!("ref: refs/heads/{}\n", branch))
            .with_context(|| format!("Failed to write '{}'.", self.head_path.display()))?;
        Ok(())
    }

    pub fn new_branch(&mut self, name: &str) -> anyhow::Result<()> {
        let path = self.refs_path.join("heads").join(name);
        if path.exists() {
            bail!("Branch '{name}' already exists.");
        }
        let start_commit = self.head_commit()?;
        fs::write(&path, start_commit.as_bytes())
            .with_context(|| format!("Failed to write '{}'.", path.display()))?;
        self.branches.insert(name.to_string(), start_commit);
        self.set_head(name)?;
        Ok(())
    }

    pub fn delete_branch(&mut self, name: &str) -> anyhow::Result<()> {
        let current = self.current_branch()?;
        if name == current {
            bail!(
                "Cannot delete the current branch '{name}'. Switch to a different branch and try again."
            );
        }
        let path = self.refs_path.join("heads").join(name);
        if !path.is_file() {
            bail!("Branch '{name}' does not exist.");
        }
        fs::remove_file(&path)
            .with_context(|| format!("Failed to delete '{}'.", path.display()))?;
        self.branches.remove(name);
        Ok(())
    }

    pub fn switch_branch(&mut self, to: &str) -> anyhow::Result<()> {
        let path = self.refs_path.join("heads").join(to);
        if !path.is_file() {
            bail!("Branch '{to}' does not exist.");
        }
        self.set_head(to)?;
        Ok(())
    }

    pub fn update_head(&mut self, commit_hash: &str) -> anyhow::Result<()> {
        let path = self.head_ref_path()?;
        fs::write(&path, commit_hash.as_bytes())
            .with_context(|| format!("Failed to write '{}'.", path.display()))?;
        let branch = self.current_branch()?;
        self.branches.insert(branch, commit_hash.to_string());
        Ok(())
    }

    pub fn branch_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.branches.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn format_branches(&self) -> anyhow::Result<String> {
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

    pub fn list_branches(&self) -> anyhow::Result<Vec<String>> {
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

    pub fn exists(&self, name: &str) -> bool {
        self.refs_path.join("heads").join(name).is_file()
    }

    pub fn get_branch_head(&self, branch: &str) -> anyhow::Result<Option<String>> {
        Ok(self.branches.get(branch).cloned())
    }
}
