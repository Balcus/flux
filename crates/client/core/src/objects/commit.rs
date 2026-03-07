use super::object_type::ObjectType;
use crate::objects::object::Object;
use crate::utils;
use chrono::Local;
use std::any::Any;
use std::fmt;

pub struct Commit {
    parent_hash: Option<String>,
    pub tree_hash: String,
    pub content: Vec<u8>,
}

impl Commit {
    pub fn new(
        tree_hash: String,
        user_name: String,
        user_email: String,
        parent_hash: Option<String>,
        message: String,
    ) -> Self {
        let now = Local::now();
        let parent_line = match parent_hash {
            Some(ref h) => format!("parent {}\n", h),
            None => String::new(),
        };
        let content = format!(
            "tree {}\n{}author {} <{}> {} {}\ncommitter {} <{}> {} {}\n\n{}",
            tree_hash,
            parent_line,
            user_name,
            user_email,
            now.timestamp(),
            now.format("%z"),
            user_name,
            user_email,
            now.timestamp(),
            now.format("%z"),
            message
        )
        .as_bytes()
        .to_owned();

        Self {
            content,
            parent_hash,
            tree_hash,
        }
    }

    pub fn from_content(content: Vec<u8>) -> Self {
        let content_str = String::from_utf8_lossy(&content);

        let mut tree_hash = String::new();
        let mut parent_hash = None;

        for line in content_str.lines() {
            if line.starts_with("tree ") {
                tree_hash = line.strip_prefix("tree ").unwrap_or("").to_string();
            } else if line.starts_with("parent ") {
                parent_hash = Some(line.strip_prefix("parent ").unwrap_or("").to_string());
            }
        }

        Self {
            content,
            parent_hash,
            tree_hash,
        }
    }

    pub fn to_string(&self) -> String {
        String::from_utf8(self.content.clone()).expect("Could not convert commit content to string")
    }

    pub fn parent_hash(&self) -> Option<&str> {
        self.parent_hash.as_deref()
    }
}

impl Object for Commit {
    fn object_type(&self) -> ObjectType {
        ObjectType::Commit
    }

    fn id(&self) -> String {
        let header = format!("commit {}\0", self.content.len());
        let mut full = Vec::new();
        full.extend_from_slice(header.as_bytes());
        full.extend_from_slice(&self.content);
        utils::hash(&full)
    }

    fn serialize(&self) -> Vec<u8> {
        let header = format!("commit {}\0", self.content.len());
        let mut full = Vec::new();
        full.extend_from_slice(header.as_bytes());
        full.extend_from_slice(&self.content);
        utils::compress(&full)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn content(&self) -> Vec<u8> {
        self.content.clone()
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
