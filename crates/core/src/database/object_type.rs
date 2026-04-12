use std::fmt;

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
            Self::Tag => "tag",
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
