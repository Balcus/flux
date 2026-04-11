use crate::diff::{
    edit::{Edit, EditType},
    hunk::Hunk,
    line::Line,
};

pub struct Mayers {
    a: Vec<Line>,
    b: Vec<Line>,
}

impl Mayers {
    pub fn new(a: Vec<Line>, b: Vec<Line>) -> Self {
        Self { a, b }
    }

    pub fn diff_hunks(a: &str, b: &str) -> Vec<Hunk> {
        let a_lines = Line::lines(a);
        let b_lines = Line::lines(b);

        let mayers = Mayers::new(a_lines, b_lines);
        Hunk::filter(mayers.diff())
    }

    pub fn diff(&self) -> Vec<Edit> {
        let mut edits = Vec::new();

        self.backtrack(|prev_x, prev_y, x, y| {
            let a_idx = prev_x as usize;
            let b_idx = prev_y as usize;

            if x == prev_x {
                edits.push(Edit {
                    edit_type: EditType::Insertion,
                    a_line: None,
                    b_line: Some(self.b[b_idx].clone()),
                });
            } else if y == prev_y {
                edits.push(Edit {
                    edit_type: EditType::Deletion,
                    a_line: Some(self.a[a_idx].clone()),
                    b_line: None,
                });
            } else {
                edits.push(Edit {
                    edit_type: EditType::Equal,
                    a_line: Some(self.a[a_idx].clone()),
                    b_line: Some(self.b[b_idx].clone()),
                });
            }
        });

        edits.reverse();
        edits
    }

    fn backtrack<F>(&self, mut yield_move: F)
    where
        F: FnMut(i32, i32, i32, i32),
    {
        let mut x = self.a.len() as i32;
        let mut y = self.b.len() as i32;

        let trace = self.shortest_edit();
        let offset = (self.a.len() + self.b.len()) as i32;

        // tterate from last depth back to inital state 0
        for d in (0..trace.len()).rev() {
            let v = &trace[d];
            let d_i32 = d as i32;
            let k = x - y;

            // determine if we moved right or down
            let prev_k = if k == -d_i32
                || (k != d_i32 && v[(offset + k - 1) as usize] < v[(offset + k + 1) as usize])
            {
                k + 1 // came from above (down move)
            } else {
                k - 1 // came from the left (right move)
            };

            let prev_x = v[(offset + prev_k) as usize];
            let prev_y = prev_x - prev_k;

            // diagonal moves (no changes)
            while x > prev_x && y > prev_y {
                yield_move(x - 1, y - 1, x, y);
                x -= 1;
                y -= 1;
            }

            // find the edit done for each depth
            if d > 0 {
                yield_move(prev_x, prev_y, x, y);
            }

            x = prev_x;
            y = prev_y;
        }
    }

    fn shortest_edit(&self) -> Vec<Vec<i32>> {
        let n = self.a.len() as i32;
        let m = self.b.len() as i32;
        let max = n + m;

        if max == 0 {
            return vec![];
        }

        // The array will be split so that:
        // index 0 ->  k = -max
        // index offset -> k = 0
        // index 2max -> k = max
        // The array v will store the furthest x coordinate reached on each diagonal k
        let mut v = vec![0; (2 * max + 1) as usize];
        let offset = max;

        // we will need the trace to backtrack our steps and see wheter we choose addition or deletion for each step
        let mut trace = Vec::new();

        // depth will go to most n + m meaning that at worst we would have to delete all of a and insert all of b
        for d in 0..=max {
            // store the previous state array before we do the iteration for the next depth
            trace.push(v.clone());

            // k will be between -d and d going by 2 as the parity of d and k must match
            for k in (-d..=d).step_by(2) {
                // to reach position k means to either go down form k + 1 (addind a character)
                // or by going right from k - 1 (removing a character)
                let mut x = if k == -d
                    || (k != d && v[(offset + k - 1) as usize] < v[(offset + k + 1) as usize])
                {
                    v[(offset + k + 1) as usize] // move down (moving down increments y and leaves x the same so we just copy the above neighbour)
                } else {
                    v[(offset + k - 1) as usize] + 1 // move right (increments x)
                };

                let mut y = x - k;

                // if the letters from a and b match we move diagonally
                while x < n && y < m && (self.a[x as usize].text == self.b[y as usize].text) {
                    x += 1;
                    y += 1;
                }

                v[(offset + k) as usize] = x;

                // we got a match between the 2 words so we can return
                if x >= n && y >= m {
                    return trace;
                }
            }
        }
        trace
    }
}
