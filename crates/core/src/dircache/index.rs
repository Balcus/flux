use crate::dircache::checksum_reader::ChecksumReader;
use crate::dircache::checksum_writer::ChecksumWriter;
use crate::dircache::index_entry::IndexEntry;
use crate::utils::lockfile::Lockfile;
use std::collections::BTreeMap;
use std::fs::{File, Metadata};
use std::path::PathBuf;

pub struct Index {
    pub path: PathBuf,
    pub entries: BTreeMap<(String, u8), IndexEntry>,
    pub lock: Lockfile,
    changed: bool,
}

impl Index {
    pub fn new(path: PathBuf) -> Self {
        Self {
            entries: BTreeMap::new(),
            lock: Lockfile::new(path.clone()),
            path,
            changed: false,
        }
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        self.entries.clear();
        if !self.path.exists() {
            return Ok(());
        }

        let file = File::open(&self.path)?;
        let mut reader = ChecksumReader::new(file);

        let mut header = [0u8; 12];
        reader.read(&mut header)?;

        if &header[0..4] != b"DIRC" {
            anyhow::bail!("Invalid signature");
        }
        let version = u32::from_be_bytes(header[4..8].try_into()?);
        if version != 2 {
            anyhow::bail!("Unsupported version");
        }

        let count = u32::from_be_bytes(header[8..12].try_into()?);

        for _ in 0..count {
            let entry = IndexEntry::from_reader(&mut reader)?;
            let stage = ((entry.flags >> 12) & 0x3) as u8;
            self.entries.insert((entry.path.clone(), stage), entry);
        }

        reader.verify_checksum()?;
        Ok(())
    }

    pub fn add(&mut self, pathname: String, id: String, stat: Metadata) -> anyhow::Result<()> {
        let id_bytes = hex::decode(&id)?;
        let id_array: [u8; 20] = id_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid SHA"))?;

        for stage in 1..=3 {
            self.entries.remove(&(pathname.clone(), stage));
        }

        let entry = IndexEntry::create(pathname.clone(), id_array, &stat, 0);
        self.entries.insert((pathname, 0), entry);
        self.changed = true;
        Ok(())
    }

    pub fn rm(&mut self, pathname: String) -> anyhow::Result<()> {
        for stage in 0..=3 {
            self.entries.remove(&(pathname.clone(), stage));
        }
        self.changed = true;
        Ok(())
    }

    pub fn write_updates(&mut self) -> anyhow::Result<()> {
        if !self.changed {
            self.lock.rollback()?;
            return Ok(());
        }

        self.lock.hold_for_update()?;
        let mut writer = ChecksumWriter::new(&mut self.lock);

        writer.write(b"DIRC")?;
        writer.write(&2u32.to_be_bytes())?;
        writer.write(&(self.entries.len() as u32).to_be_bytes())?;

        for entry in self.entries.values() {
            writer.write(&entry.to_bytes())?;
        }

        writer.write_checksum()?;
        self.lock.commit()?;
        self.changed = false;
        Ok(())
    }

    pub fn mark_changed(&mut self) {
        self.changed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::{TempDir, tempdir};

    fn setup_file(dir: &TempDir, name: &str, executable: bool) -> (String, String, Metadata) {
        let path = dir.path().join(name);
        fs::write(&path, "content").unwrap();

        if executable {
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }

        let metadata = fs::metadata(&path).unwrap();
        let fake_sha = "a".repeat(40);
        (name.to_string(), fake_sha, metadata)
    }

    #[test]
    fn new_index_empty() {
        let tmp = tempdir().unwrap();
        let index_path = tmp.path().join("index");
        let index = Index::new(index_path);

        assert!(index.entries.is_empty());
        assert!(!index.changed);
    }

    #[test]
    fn add_single_entry() -> anyhow::Result<()> {
        let tmp = tempdir().unwrap();
        let mut index = Index::new(tmp.path().join("index"));
        let (path, sha, meta) = setup_file(&tmp, "test.txt", false);

        index.add(path.clone(), sha.clone(), meta)?;

        assert_eq!(index.entries.len(), 1);
        let entry = index.entries.get(&(path, 0)).unwrap();
        assert_eq!(hex::encode(entry.id), sha);
        assert!(index.changed);
        Ok(())
    }

    #[test]
    fn write_and_load_cycle() -> anyhow::Result<()> {
        let tmp = tempdir().unwrap();
        let index_path = tmp.path().join("index");

        let mut index = Index::new(index_path.clone());
        let (p1, s1, m1) = setup_file(&tmp, "a.txt", false);
        let (p2, s2, m2) = setup_file(&tmp, "b.txt", true);

        index.add(p1.clone(), s1.clone(), m1)?;
        index.add(p2.clone(), s2.clone(), m2)?;
        index.write_updates()?;

        let mut new_index = Index::new(index_path);
        new_index.load()?;

        assert_eq!(new_index.entries.len(), 2);

        let entry_a = new_index.entries.get(&(p1, 0)).unwrap();
        assert_eq!(entry_a.mode, 0o100644);

        let entry_b = new_index.entries.get(&(p2, 0)).unwrap();
        assert_eq!(entry_b.mode, 0o100755);

        Ok(())
    }
}
