use crate::{diff::{edit::{Edit, EditType}, line::Line}, utils::colors::{CYAN, GREEN, RED, RESET}};
use std::fmt;

const HUNK_CONTEXT: usize = 5;

pub struct Hunk {
    pub a_start: usize,
    pub b_start: usize,
    pub edits: Vec<Edit>,
}

impl Hunk {
    pub fn filter(edits: Vec<Edit>) -> Vec<Hunk> {
        let mut hunks = Vec::new();
        let mut offset = 0;
        while offset < edits.len() {
            while offset < edits.len() && edits[offset].edit_type == EditType::Equal {
                offset += 1;
            }

            if offset >= edits.len() {
                return hunks;
            }

            let hunk_start = offset.saturating_sub(HUNK_CONTEXT);
            let a_start = edits[hunk_start].a_line.as_ref().map(|l| l.number).unwrap_or(0);
            let b_start = edits[hunk_start].b_line.as_ref().map(|l| l.number).unwrap_or(0);

            let mut hunk = Hunk {
                a_start,
                b_start,
                edits: Vec::new(),
            };

            offset = Hunk::build(&mut hunk, &edits, hunk_start);
            hunks.push(hunk);
        }
        hunks
    }

    pub fn build(hunk: &mut Hunk, all_edits: &Vec<Edit>, mut offset: usize) -> usize {
        let mut eq = 0;

        while offset < all_edits.len() {
            if all_edits[offset].edit_type == EditType::Equal {
                eq += 1;
            } else {
                eq = 0;
            }

            hunk.edits.push(all_edits[offset].clone());
            offset += 1;

            if eq == HUNK_CONTEXT * 2 + 1 {
                for _ in 0..HUNK_CONTEXT {
                    hunk.edits.pop();
                }
                return offset - HUNK_CONTEXT;
            }
        }

        offset
    }

    pub fn header(&self) -> String {
        let a_offset = self.offsets_for(|e| e.a_line.as_ref(), self.a_start);
        let b_offset = self.offsets_for(|e| e.b_line.as_ref(), self.b_start);

        format!("@@ -{} +{} @@", a_offset, b_offset)
    }

    fn offsets_for<F>(&self, get_line: F, default: usize) -> String
    where
        F: Fn(&Edit) -> Option<&Line>,
    {
        let lines: Vec<&Line> = self.edits.iter().filter_map(get_line).collect();
        let start = lines.first().map(|l| l.number).unwrap_or(default);
        format!("{},{}", start, lines.len())
    }
}

impl fmt::Display for Hunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}{}{}", CYAN, self.header(), RESET)?;
        for edit in &self.edits {
            match edit.edit_type {
                EditType::Equal => writeln!(f, "{}", edit)?,
                EditType::Insertion => writeln!(f, "{}{}{}", GREEN, edit, RESET)?,
                EditType::Deletion => writeln!(f, "{}{}{}", RED, edit, RESET)?
            }
        }
        Ok(())
    }
}
