#[derive(Clone)]
pub struct Line {
    pub number: usize,
    pub text: String,
}

impl Line {
    pub fn lines(source: &str) -> Vec<Line> {
        source
            .lines()
            .enumerate()
            .map(|(i, line)| Line {
                number: i + 1,
                text: line.to_string(),
            })
            .collect()
    }
}
