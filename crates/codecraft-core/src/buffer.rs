use lapce_xi_rope::Rope;
use crate::cursor::Cursor;

#[derive(Clone)]
pub struct DocumentBuffer {
    rope: Rope,
    cursor: Cursor,
    version: u64,
    dirty: bool,
    path: Option<String>,
}

impl DocumentBuffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::from(""),
            cursor: Cursor::default(),
            version: 0,
            dirty: false,
            path: None,
        }
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            rope: Rope::from(text),
            cursor: Cursor::default(),
            version: 0,
            dirty: false,
            path: None,
        }
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len()
    }

    pub fn len_lines(&self) -> usize {
        self.rope.line_of_offset(self.rope.len()) + 1
    }

    pub fn line(&self, line_idx: usize) -> Option<String> {
        let total = self.len_lines();
        if line_idx >= total {
            return None;
        }
        let start = self.rope.offset_of_line(line_idx);
        let end = if line_idx + 1 < total {
            self.rope.offset_of_line(line_idx + 1)
        } else {
            self.rope.len()
        };
        Some(self.rope.slice_to_cow(start..end).into_owned())
    }

    pub fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.offset_of_line(line_idx)
    }

    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.line_of_offset(char_idx)
    }

    pub fn insert_text(&mut self, char_idx: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        let clamped = char_idx.min(self.rope.len());
        let mut builder = lapce_xi_rope::DeltaBuilder::new(self.rope.len());
        builder.replace(clamped..clamped, Rope::from(text));
        let delta = builder.build();
        self.rope = delta.apply(&self.rope);
        self.version += 1;
        self.dirty = true;
    }

    pub fn delete_range(&mut self, start_char: usize, end_char: usize) {
        let max_len = self.rope.len();
        let start = start_char.min(max_len);
        let end = end_char.min(max_len);
        if start >= end {
            return;
        }
        let mut builder = lapce_xi_rope::DeltaBuilder::new(self.rope.len());
        builder.delete(start..end);
        let delta = builder.build();
        self.rope = delta.apply(&self.rope);
        self.version += 1;
        self.dirty = true;
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    pub fn snapshot(&self) -> Rope {
        self.rope.clone()
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn set_path(&mut self, path: Option<String>) {
        self.path = path;
    }

    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }
}
