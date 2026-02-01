use crate::{
    error,
    objects::{
        blob::Blob,
        commit::Commit,
        object_type::{FluxObject, ObjectType},
        tree::Tree,
    },
    utils,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, error::ObjectStoreError>;

#[derive(Debug)]
pub struct ObjectStore {
    pub path: PathBuf,
}

impl ObjectStore {
    pub fn new(flux_dir: &Path) -> Result<Self> {
        let path = flux_dir.join("objects");
        fs::create_dir(&path).map_err(|e| error::IoError::create_error(&path, e))?;
        Ok(Self { path })
    }

    pub fn load(flux_dir: &Path) -> Result<Self> {
        let path = flux_dir.join("objects");
        if !path.exists() {
            return Err(error::IoError::missing_error(&path).into());
        }
        Ok(Self { path })
    }

    pub fn store(&self, object: &dyn FluxObject) -> Result<()> {
        self.store_object(&object.hash(), &object.serialize())?;
        Ok(())
    }

    pub fn retrieve_object(&self, hash: &str) -> Result<Box<dyn FluxObject>> {
        let object = self.read_object(hash)?;
        match object.object_type {
            ObjectType::Blob => Ok(Box::new(Blob::from_content(object.decompressed_content))),
            ObjectType::Tree => Ok(Box::new(Tree::from_content(object.decompressed_content))),
            ObjectType::Commit => Ok(Box::new(Commit::from_content(object.decompressed_content))),
            ObjectType::Tag => Err(error::ObjectError::Unsupported {
                object_type: object.object_type.to_string(),
            }
            .into()),
        }
    }

    pub fn raw_content(&self, hash: &str) -> Result<Vec<u8>> {
        let (dir, file) = hash.split_at(2);
        let object_path = self.path.join(dir).join(file);

        let compressed_content =
            fs::read(&object_path).map_err(|e| error::IoError::read_error(&object_path, e))?;

        Ok(compressed_content)
    }

    /// Reads a git object from `.flux/objects` given its hash.
    ///
    /// Locates the object on disk, decompresses it, parses the header and validates the content size.  
    /// Returns a `GenericObject` containing:
    /// - `object_type`
    /// - `size`
    /// - `decompressed_content`
    fn read_object(&self, hash: &str) -> Result<utils::GenericObject> {
        let (dir, file) = hash.split_at(2);
        let object_path = self.path.join(dir).join(file);

        let compressed_content =
            fs::read(&object_path).map_err(|e| error::IoError::read_error(&object_path, e))?;
        let decompressed = utils::decompress(compressed_content);

        let null_pos = decompressed
            .iter()
            .position(|&b| b == b'\0')
            .ok_or_else(|| error::ObjectError::InvalidFormat {
                path: object_path.clone(),
                hash: hash.to_string(),
            })?;

        let header = String::from_utf8(decompressed[..null_pos].to_vec()).map_err(|_e| {
            error::ObjectError::InvalidFormat {
                path: object_path.clone(),
                hash: hash.to_string(),
            }
        })?;

        let parts: Vec<&str> = header.split(' ').collect();

        if parts.len() != 2 {
            return Err(error::ObjectError::InvalidFormat {
                path: object_path.clone(),
                hash: hash.to_string(),
            }
            .into());
        }

        let object_type = match parts[0] {
            "blob" => ObjectType::Blob,
            "tree" => ObjectType::Tree,
            "commit" => ObjectType::Commit,
            _ => {
                return Err(error::ObjectError::Unsupported {
                    object_type: parts[0].to_string(),
                }
                .into());
            }
        };

        let size: usize = parts[1]
            .parse()
            .map_err(|_e| error::ObjectError::InvalidFormat {
                path: object_path.clone(),
                hash: hash.to_string(),
            })?;
        let decompressed_content = decompressed[null_pos + 1..].to_vec();

        if decompressed_content.len() != size {
            return Err(error::ObjectError::SizeMismatch {
                path: object_path.clone(),
                hash: hash.to_string(),
                expected: size,
                got: decompressed_content.len(),
            }
            .into());
        }

        Ok(utils::GenericObject {
            object_type,
            size,
            decompressed_content,
        })
    }

    /// Writes a git object to the `.flux/objects` directory, given the object's `compressed` contents
    fn store_object(&self, hash: &str, compressed_data: &[u8]) -> Result<()> {
        let (dir, file) = hash.split_at(2);
        let object_dir = self.path.join(dir);
        fs::create_dir_all(&object_dir)
            .map_err(|e| error::IoError::create_error(&object_dir, e))?;
        let object_path = self.path.join(dir).join(file);

        let temp_path: std::path::PathBuf = object_path.with_extension("tmp");
        fs::write(&temp_path, compressed_data)
            .map_err(|e| error::IoError::write_error(&object_dir, e))?;
        fs::rename(&temp_path, &object_path)
            .map_err(|e| error::IoError::rename_error(&temp_path, &object_path, e))?;

        Ok(())
    }
}
