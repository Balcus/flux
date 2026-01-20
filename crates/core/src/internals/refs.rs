use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

pub struct Refs {
    pub refs_path: PathBuf,
    pub branches: HashMap<String, String>,
    pub head_path: PathBuf,
}

impl Refs {
    fn parse_head_ref(head_contents: &str) -> anyhow::Result<String> {
        let s = head_contents.trim();
        let r = s
            .strip_prefix("ref: ")
            .ok_or_else(|| anyhow::anyhow!("Invalid HEAD format: {}", s))?;
        if !r.starts_with("refs/heads/") {
            anyhow::bail!("Invalid HEAD ref (expected refs/heads/*): {}", r);
        }
        Ok(r.to_string())
    }

    pub fn new(flux_dir: &Path) -> Self {
        let refs_path = flux_dir.join("refs");
        let head_path = flux_dir.join("HEAD");

        fs::create_dir_all(&refs_path.join("heads"))
            .expect("Failed to create refs/heads directory");

        fs::write(refs_path.join("heads/main"), "").expect("Failed to create main branch");
        fs::write(&head_path, "ref: refs/heads/main\n").expect("Failed to init HEAD");

        let mut branches = HashMap::new();
        branches.insert("main".to_string(), "".to_string());

        Self {
            refs_path,
            branches,
            head_path,
        }
    }

    pub fn load(flux_dir: &Path) -> Self {
        let refs_path = flux_dir.join("refs");
        let heads_path = refs_path.join("heads");

        if !refs_path.is_dir() || !heads_path.is_dir() {
            panic!("Missing refs directories");
        }

        let heads = fs::read_dir(&heads_path).expect("Failed to read refs/heads");
        let mut map: HashMap<String, String> = HashMap::new();

        for file in heads {
            let file = file.expect("Failed to read file");
            let name = file.file_name().to_string_lossy().into_owned();
            let head = fs::read_to_string(file.path())
                .expect(&format!("Failed to read head: {:?}", file.path()));
            map.insert(name, head.trim().to_string());
        }

        Self {
            refs_path,
            branches: map,
            head_path: flux_dir.join("HEAD"),
        }
    }

    pub fn head_ref(&self) -> anyhow::Result<String> {
        let raw = fs::read_to_string(&self.head_path)?;
        Self::parse_head_ref(&raw)
    }

    pub fn current_branch(&self) -> anyhow::Result<String> {
        let head_ref = self.head_ref()?;
        let name = head_ref
            .strip_prefix("refs/heads/")
            .ok_or_else(|| anyhow::anyhow!("Invalid HEAD ref: {}", head_ref))?;
        Ok(name.to_string())
    }

    pub fn head_ref_path(&self) -> anyhow::Result<PathBuf> {
        let head_ref = self.head_ref()?;
        let rel = head_ref
            .strip_prefix("refs/")
            .ok_or_else(|| anyhow::anyhow!("Invalid HEAD ref: {}", head_ref))?;
        Ok(self.refs_path.join(rel))
    }

    pub fn head_commit(&self) -> anyhow::Result<String> {
        let branch_path = self.head_ref_path()?;
        Ok(fs::read_to_string(branch_path)
            .unwrap_or_default()
            .trim()
            .to_string())
    }

    pub fn set_head(&self, branch: &str) {
        fs::write(&self.head_path, format!("ref: refs/heads/{}\n", branch))
            .expect("Failed to set HEAD");
    }

    pub fn new_branch(&mut self, name: &str) {
        let path = self.refs_path.join("heads").join(name);

        if path.exists() {
            panic!("Branch already exists");
        }

        let start_commit = self.head_commit().expect("Failed to read contents of HEAD");
        fs::write(&path, start_commit.as_bytes()).expect("Failed to write branch head");

        self.branches.insert(name.to_string(), start_commit);
        self.set_head(name);
    }

    pub fn delete_branch(&mut self, name: &str) -> anyhow::Result<()> {
        let current = self.current_branch()?;
        if name == current {
            anyhow::bail!("Cannot delete the current branch '{}'", name);
        }

        let path = self.refs_path.join("heads").join(name);
        if !path.is_file() {
            anyhow::bail!("Branch '{}' does not exist", name);
        }

        fs::remove_file(&path)?;
        self.branches.remove(name);
        Ok(())
    }

    pub fn switch_branch(&mut self, to: &str) -> anyhow::Result<()> {
        let path = self.refs_path.join("heads").join(to);
        if !path.is_file() {
            anyhow::bail!("Branch '{}' does not exist", to);
        }
        self.set_head(to);
        Ok(())
    }

    pub fn update_head(&mut self, commit_hash: &str) -> anyhow::Result<()> {
        let path = self.head_ref_path()?;
        fs::write(&path, commit_hash.as_bytes())?;

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
}
