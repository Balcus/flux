use crate::diff::line::Line;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EditType {
    Insertion,
    Deletion,
    Equal,
}

#[derive(Clone)]
pub struct Edit {
    pub edit_type: EditType,
    pub a_line: Option<Line>,
    pub b_line: Option<Line>,
}

impl Edit {
    fn symbol(&self) -> &'static str {
        match self.edit_type {
            EditType::Insertion => "+",
            EditType::Deletion => "-",
            EditType::Equal => "",
        }
    }
}

impl fmt::Display for Edit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let line = self.a_line.as_ref().or(self.b_line.as_ref()).unwrap();
        write!(f, "{} {}", self.symbol(), line.text)
    }
}
