use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct Lockfile {
    file_path: PathBuf,
    lock_path: PathBuf,
    lock: Option<File>,
}

impl Lockfile {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let file_path = path.as_ref().to_path_buf();
        let mut lock_path = file_path.clone();
        lock_path.set_extension("lock");
        Self {
            file_path,
            lock_path,
            lock: None,
        }
    }

    pub fn hold_for_update(&mut self) -> io::Result<()> {
        if self.lock.is_none() {
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&self.lock_path)?;
            self.lock = Some(file);
        }
        Ok(())
    }

    pub fn commit(&mut self) -> io::Result<()> {
        self.lock = None;
        fs::rename(&self.lock_path, &self.file_path)?;
        Ok(())
    }

    pub fn rollback(&mut self) -> io::Result<()> {
        self.lock = None;
        if self.lock_path.exists() {
            fs::remove_file(&self.lock_path)?;
        }
        Ok(())
    }
}

impl Write for Lockfile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.lock.as_mut().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.lock.as_mut().unwrap().flush()
    }
}
