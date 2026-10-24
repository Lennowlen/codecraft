use crate::cursor::Cursor;

#[derive(Debug, Clone)]
pub struct EditOp {
    pub start_byte: usize,
    pub old_text: String,
    pub new_text: String,
}

#[derive(Clone)]
pub struct DocumentBuffer {
    rope: ropey::Rope,
    cursor: Cursor,
    undo_stack: Vec<EditOp>,
    redo_stack: Vec<EditOp>,
    version: u64,
    dirty: bool,
    path: Option<String>,
}

impl DocumentBuffer {
    pub fn new() -> Self {
        Self {
            rope: ropey::Rope::new(),
            cursor: Cursor::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            version: 0,
            dirty: false,
            path: None,
        }
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            rope: ropey::Rope::from_str(text),
            cursor: Cursor::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            version: 0,
            dirty: false,
            path: None,
        }
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx >= self.rope.len_lines() {
            None
        } else {
            Some(self.rope.line(line_idx).to_string())
        }
    }

    pub fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx)
    }

    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx)
    }

    pub fn insert_text(&mut self, char_idx: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        let clamped = char_idx.min(self.rope.len_chars());
        self.rope.insert(clamped, text);
        self.undo_stack.push(EditOp {
            start_byte: clamped,
            old_text: String::new(),
            new_text: text.to_string(),
        });
        self.redo_stack.clear();
        self.version += 1;
        self.dirty = true;
    }

    pub fn delete_range(&mut self, start_char: usize, end_char: usize) {
        let max_len = self.rope.len_chars();
        let start = start_char.min(max_len);
        let end = end_char.min(max_len);
        if start >= end {
            return;
        }
        let removed = self.rope.slice(start..end).to_string();
        self.rope.remove(start..end);
        self.undo_stack.push(EditOp {
            start_byte: start,
            old_text: removed,
            new_text: String::new(),
        });
        self.redo_stack.clear();
        self.version += 1;
        self.dirty = true;
    }

    pub fn undo(&mut self) -> bool {
        if let Some(op) = self.undo_stack.pop() {
            let start = op.start_byte.min(self.rope.len_chars());
            if !op.new_text.is_empty() {
                let end = (start + op.new_text.chars().count()).min(self.rope.len_chars());
                self.rope.remove(start..end);
            }
            if !op.old_text.is_empty() {
                self.rope.insert(start, &op.old_text);
            }
            self.redo_stack.push(op);
            self.version += 1;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(op) = self.redo_stack.pop() {
            let start = op.start_byte.min(self.rope.len_chars());
            if !op.old_text.is_empty() {
                let end = (start + op.old_text.chars().count()).min(self.rope.len_chars());
                self.rope.remove(start..end);
            }
            if !op.new_text.is_empty() {
                self.rope.insert(start, &op.new_text);
            }
            self.undo_stack.push(op);
            self.version += 1;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    pub fn snapshot(&self) -> ropey::Rope {
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
