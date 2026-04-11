use crate::{database::object::Object, utils};

use super::object_type::ObjectType;
use std::{any::Any, fmt, str::from_utf8};

pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.as_bytes().to_owned(),
        }
    }

    pub fn as_string(&self) -> String {
        String::from_utf8(self.data.clone()).expect("Could not read blob contents to string")
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data }
    }

    fn serialized(&self) -> Vec<u8> {
        let header = format!("blob {}\0", self.data.len());
        let mut full = Vec::new();
        full.extend_from_slice(header.as_bytes());
        full.extend_from_slice(&self.data);
        full
    }
}

impl Object for Blob {
    fn object_type(&self) -> ObjectType {
        ObjectType::Blob
    }

    fn id(&self) -> String {
        utils::hash(&self.serialized())
    }

    fn serialize(&self) -> Vec<u8> {
        self.serialized()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn content(&self) -> Vec<u8> {
        self.data.clone()
    }
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = from_utf8(&self.data).map_err(|_| fmt::Error)?;
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use crate::database::{blob::Blob, object::Object, object_type::ObjectType};
    use anyhow::Result;

    #[test]
    fn blob_from_data() -> Result<()> {
        let content = "hello world\n";
        let blob = Blob::new(content);

        assert_eq!(blob.data, content.as_bytes());
        assert_eq!(blob.object_type(), ObjectType::Blob);
        assert_eq!(blob.as_string(), "hello world\n".to_string());

        let serialized = blob.serialize();
        let header = format!("blob {}\0", content.len());
        let mut bytes = Vec::new();
        bytes.extend_from_slice(header.as_bytes());
        bytes.extend_from_slice(content.as_bytes());

        assert_eq!(serialized, bytes);
        assert_eq!(
            blob.id(),
            "3b18e512dba79e4c8300dd08aeb37f8e728b8dad".to_string()
        );

        Ok(())
    }

    #[test]
    fn blob_from_empty() -> Result<()> {
        let blob = Blob::new("");
        assert_eq!(blob.data, b"");
        assert_eq!(blob.as_string(), "");
        assert_eq!(blob.object_type(), ObjectType::Blob);
        assert_eq!(
            blob.id(),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391".to_string()
        );
        Ok(())
    }
}
