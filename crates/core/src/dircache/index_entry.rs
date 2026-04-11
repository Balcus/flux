use std::cmp::min;
use std::fs::Metadata;
use std::io::Read;
use std::os::unix::fs::MetadataExt;

use crate::dircache::checksum_reader::ChecksumReader;

const REGULAR_MODE: u32 = 0o100644;
const EXECUTABLE_MODE: u32 = 0o100755;
const MAX_PATH_SIZE: u16 = 0xfff;

pub struct IndexEntry {
    pub ctime_s: u32,
    pub ctime_ns: u32,
    pub mtime_s: u32,
    pub mtime_ns: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
    pub id: [u8; 20],
    pub flags: u16,
    pub path: String,
}

impl IndexEntry {
    pub fn create(path: String, id: [u8; 20], stat: &Metadata, stage: u8) -> Self {
        let mode = if stat.mode() & 0o111 != 0 {
            EXECUTABLE_MODE
        } else {
            REGULAR_MODE
        };

        let len = min(path.len() as u16, MAX_PATH_SIZE);
        let flags = ((stage as u16 & 0x3) << 12) | len;

        Self {
            ctime_s: stat.ctime() as u32,
            ctime_ns: stat.ctime_nsec() as u32,
            mtime_s: stat.mtime() as u32,
            mtime_ns: stat.mtime_nsec() as u32,
            dev: stat.dev() as u32,
            ino: stat.ino() as u32,
            mode,
            uid: stat.uid(),
            gid: stat.gid(),
            size: stat.size() as u32,
            id,
            flags,
            path,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64 + self.path.len());
        buf.extend_from_slice(&self.ctime_s.to_be_bytes());
        buf.extend_from_slice(&self.ctime_ns.to_be_bytes());
        buf.extend_from_slice(&self.mtime_s.to_be_bytes());
        buf.extend_from_slice(&self.mtime_ns.to_be_bytes());
        buf.extend_from_slice(&self.dev.to_be_bytes());
        buf.extend_from_slice(&self.ino.to_be_bytes());
        buf.extend_from_slice(&self.mode.to_be_bytes());
        buf.extend_from_slice(&self.uid.to_be_bytes());
        buf.extend_from_slice(&self.gid.to_be_bytes());
        buf.extend_from_slice(&self.size.to_be_bytes());
        buf.extend_from_slice(&self.id);
        buf.extend_from_slice(&self.flags.to_be_bytes());
        buf.extend_from_slice(self.path.as_bytes());
        let pad_len = 8 - (buf.len() % 8);
        buf.extend(std::iter::repeat(0).take(pad_len));
        buf
    }

    pub fn from_reader<R: Read>(reader: &mut ChecksumReader<R>) -> anyhow::Result<Self> {
        let mut meta = [0u8; 40];
        reader.read(&mut meta)?;

        let mut id = [0u8; 20];
        reader.read(&mut id)?;

        let mut flags_buf = [0u8; 2];
        reader.read(&mut flags_buf)?;
        let flags = u16::from_be_bytes(flags_buf);

        let mut path_bytes = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            reader.read(&mut byte)?;
            if byte[0] == 0 {
                break;
            }
            path_bytes.push(byte[0]);
        }
        let path = String::from_utf8(path_bytes)?;

        let entry_len = 62 + path.len() + 1;
        let pad_len = (8 - (entry_len % 8)) % 8;
        if pad_len > 0 {
            let mut pad = vec![0u8; pad_len];
            reader.read(&mut pad)?;
        }

        Ok(Self {
            ctime_s: u32::from_be_bytes(meta[0..4].try_into()?),
            ctime_ns: u32::from_be_bytes(meta[4..8].try_into()?),
            mtime_s: u32::from_be_bytes(meta[8..12].try_into()?),
            mtime_ns: u32::from_be_bytes(meta[12..16].try_into()?),
            dev: u32::from_be_bytes(meta[16..20].try_into()?),
            ino: u32::from_be_bytes(meta[20..24].try_into()?),
            mode: u32::from_be_bytes(meta[24..28].try_into()?),
            uid: u32::from_be_bytes(meta[28..32].try_into()?),
            gid: u32::from_be_bytes(meta[32..36].try_into()?),
            size: u32::from_be_bytes(meta[36..40].try_into()?),
            id,
            flags,
            path,
        })
    }
}
