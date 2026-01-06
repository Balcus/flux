use crate::{objects::object_type::FluxObject, utils};

use super::object_type::ObjectType;
use std::{any::Any, fs, path::Path};

pub struct Blob {
    content: Vec<u8>,
}

impl Blob {
    pub fn new(file: &Path) -> Self {
        let content = fs::read(file).expect("Could not read file content");
        Self { content }
    }
    
    pub fn to_string(&self) -> String {
        String::from_utf8(self.content.clone())
            .expect("Could not read blob contents to string")
    }

    pub fn from_content(content: Vec<u8>) -> Self {
        Self { content }
    }
}

impl FluxObject for Blob {
    fn object_type(&self) -> ObjectType {
        ObjectType::Blob
    }
    
    fn hash(&self) -> String {
        let header = format!("blob {}\0", self.content.len());
        let mut full = Vec::new();
        full.extend_from_slice(header.as_bytes());
        full.extend_from_slice(&self.content);
        utils::hash(&full)
    }
    
    fn serialize(&self) -> Vec<u8> {
        let header = format!("blob {}\0", self.content.len());
        let mut full = Vec::new();
        full.extend_from_slice(header.as_bytes());
        full.extend_from_slice(&self.content);
        utils::compress(&full)
    }
    
    fn print(&self) {
        println!("{}", self.to_string());
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn content(&self) -> Vec<u8> {
        self.content.clone()
    }
}
