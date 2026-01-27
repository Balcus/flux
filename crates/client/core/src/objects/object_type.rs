use std::{any::Any, fmt};

pub trait FluxObject {
    fn object_type(&self) -> ObjectType;
    fn hash(&self) -> String;
    fn serialize(&self) -> Vec<u8>;
    fn print(&self);
    fn as_any(&self) -> &dyn Any;
    fn content(&self) -> Vec<u8>;
}

#[derive(PartialEq, Debug)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl ObjectType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Blob => "blob",
            Self::Tree => "tree",
            Self::Commit => "commit",
            Self::Tag => "tag"
        }
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjectType::Blob => write!(f, "Blob"),
            ObjectType::Tree => write!(f, "Tree"),
            ObjectType::Commit => write!(f, "Commit"),
            ObjectType::Tag => write!(f, "Tag"),
        }
    }
}