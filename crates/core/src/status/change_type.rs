#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}

impl ChangeType {
    pub fn to_string(&self) -> String {
        match self {
            Self::Added => "Added".to_string(),
            Self::Modified => "Modified".to_string(),
            Self::Deleted => "Deleted".to_string(),
        }
    }
}
