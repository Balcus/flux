use std::{fs, path::PathBuf, io::Read};
use anyhow::Context;
use flate2::read::ZlibDecoder;

pub trait GitObject {
    fn cat_file(worktree: PathBuf, hash: String) -> Result<String, anyhow::Error>;
    fn hash_object(worktree: PathBuf, path: String, write: bool) -> Result<String, anyhow::Error>;

    fn read_object(worktree: PathBuf, hash: String) -> Result<(String, usize, Vec<u8>), anyhow::Error> {
        let (dir, file) = hash.split_at(2);
        let path = PathBuf::from(format!(".git/objects/{}/{}", dir, file));
        let path = worktree.join(path);

        let compressed = fs::read(path)
            .with_context(|| format!("Could not read git object for hash: {}", &hash))?;

        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut byte = [0u8; 1];
        let mut header = Vec::new();

        loop {
            decoder.read_exact(&mut byte)?;
            if byte[0] == b'\0' {
                break;
            }
            header.push(byte[0]);
            
            if header.len() > 100 {
                return Err(anyhow::anyhow!("Invalid git object header"));
            }
        }

        let header_str = String::from_utf8(header)
            .with_context(|| "Could not parse git object header")?;
        let header_parts: Vec<&str> = header_str.split(' ').collect();
        if header_parts.len() != 2 {
            anyhow::bail!("Invalid git object header");
        }

        let object_type = header_parts[0].to_string();
        let size: usize = header_parts[1].parse()?;

        let mut content = vec![0u8; size];
        decoder.read_exact(&mut content)?;

        Ok((object_type, size, content))
    }
}