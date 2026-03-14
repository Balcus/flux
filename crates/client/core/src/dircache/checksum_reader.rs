use sha1::{Digest, Sha1};
use std::io::{self, Read};

pub struct ChecksumReader<R: Read> {
    inner: R,
    hasher: Sha1,
}

impl<R: Read> ChecksumReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            hasher: Sha1::new(),
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)?;
        self.hasher.update(buf);
        Ok(())
    }

    pub fn verify_checksum(&mut self) -> anyhow::Result<()> {
        let mut actual_checksum = [0u8; 20];
        self.inner.read_exact(&mut actual_checksum)?;

        let expected_checksum = self.hasher.clone().finalize();
        if actual_checksum != expected_checksum.as_slice() {
            anyhow::bail!("Checksum mismatch");
        }
        Ok(())
    }
}
