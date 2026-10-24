#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
    pub char_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorRange {
    pub start: Cursor,
    pub end: Cursor,
}

impl Cursor {
    pub fn new(line: usize, col: usize, char_offset: usize) -> Self {
        Self { line, col, char_offset }
    }
}
