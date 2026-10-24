use crate::buffer::DocumentBuffer;
use crate::syntax::{LanguageId, SyntaxEngine};

pub struct EditorTab {
    pub id: usize,
    pub title: String,
    pub path: Option<String>,
    pub buffer: DocumentBuffer,
    pub syntax: SyntaxEngine,
    pub is_active: bool,
}

pub struct WorkspaceManager {
    tabs: Vec<EditorTab>,
    active_tab_idx: usize,
    next_id: usize,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_idx: 0,
            next_id: 1,
        }
    }

    pub fn open_file(&mut self, title: String, path: Option<String>, content: &str) -> usize {
        let ext = path.as_deref().and_then(|p| p.split('.').last()).unwrap_or("");
        let lang = LanguageId::from_extension(ext);
        let mut buffer = DocumentBuffer::from_str(content);
        buffer.set_path(path.clone());

        let id = self.next_id;
        self.next_id += 1;

        let tab = EditorTab {
            id,
            title,
            path,
            buffer,
            syntax: SyntaxEngine::new(lang),
            is_active: true,
        };

        for t in &mut self.tabs {
            t.is_active = false;
        }

        self.tabs.push(tab);
        self.active_tab_idx = self.tabs.len() - 1;
        id
    }

    pub fn active_tab(&self) -> Option<&EditorTab> {
        self.tabs.get(self.active_tab_idx)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut EditorTab> {
        self.tabs.get_mut(self.active_tab_idx)
    }

    pub fn tabs(&self) -> &[EditorTab] {
        &self.tabs
    }

    pub fn select_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            for (i, t) in self.tabs.iter_mut().enumerate() {
                t.is_active = i == idx;
            }
            self.active_tab_idx = idx;
        }
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.tabs.remove(idx);
            if self.tabs.is_empty() {
                self.active_tab_idx = 0;
            } else if self.active_tab_idx >= self.tabs.len() {
                self.active_tab_idx = self.tabs.len() - 1;
                self.tabs[self.active_tab_idx].is_active = true;
            }
        }
    }
}
