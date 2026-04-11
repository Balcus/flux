use sha1::{Digest, Sha1};
use std::io::{self, Write};

pub struct ChecksumWriter<'a, W: Write> {
    inner: &'a mut W,
    hasher: Sha1,
}

impl<'a, W: Write> ChecksumWriter<'a, W> {
    pub fn new(inner: &'a mut W) -> Self {
        Self {
            inner,
            hasher: Sha1::new(),
        }
    }

    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.inner.write_all(data)?;
        self.hasher.update(data);
        Ok(())
    }

    pub fn write_checksum(&mut self) -> io::Result<()> {
        let actual_hash = self.hasher.clone().finalize();
        self.inner.write_all(&actual_hash)
    }
}
