use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct DiskUtils {
    pub objects_path: PathBuf,
}

impl DiskUtils {
    pub fn new(objects_path: PathBuf) -> Self {
        Self { objects_path }
    }

    pub fn write_object(&self, id: &str, data: &[u8]) -> anyhow::Result<()> {
        let (dir, file) = id.split_at(2);
        let object_dir = self.objects_path.join(dir);
        let object_path = object_dir.join(file);

        if object_path.exists() {
            return Ok(());
        }

        fs::create_dir_all(&object_dir)?;
        let temp_path = object_path.with_extension("tmp");
        let file_handle = fs::File::create(&temp_path)?;
        let mut encoder = ZlibEncoder::new(file_handle, Compression::default());
        encoder.write_all(data)?;
        encoder.finish()?;
        fs::rename(&temp_path, &object_path)?;

        Ok(())
    }

    pub fn read_raw(&self, id: &str) -> std::io::Result<Vec<u8>> {
        let path = self.objects_path.join(&id[0..2]).join(&id[2..]);
        let file = fs::File::open(path)?;
        let mut decoder = ZlibDecoder::new(file);
        let mut buffer = Vec::new();
        decoder.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}
