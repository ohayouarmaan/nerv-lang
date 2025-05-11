#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Position {
    pub line: usize,
    pub column: usize
}

#[allow(dead_code)]
impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column
        }
    }
}
