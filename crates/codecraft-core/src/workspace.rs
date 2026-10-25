use crate::buffer::DocumentBuffer;
use crate::syntax::{LanguageId, SyntaxEngine};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct EditorTab {
    pub id: usize,
    pub title: String,
    pub path: Option<String>,
    pub buffer: DocumentBuffer,
    pub syntax: SyntaxEngine,
    pub is_active: bool,
    pub is_dirty: bool,
}

#[derive(Debug, Clone)]
pub struct FileTreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileTreeNode>,
    pub is_expanded: bool,
}

pub struct WorkspaceManager {
    tabs: Vec<EditorTab>,
    active_tab_idx: usize,
    next_id: usize,
    root_dir: PathBuf,
    file_tree: Vec<FileTreeNode>,
}

impl WorkspaceManager {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root_dir = root.into();
        let mut mgr = Self {
            tabs: Vec::new(),
            active_tab_idx: 0,
            next_id: 1,
            root_dir: root_dir.clone(),
            file_tree: Vec::new(),
        };
        mgr.refresh_file_tree();
        mgr
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn set_root_dir(&mut self, root: impl Into<PathBuf>) {
        self.root_dir = root.into();
        self.refresh_file_tree();
    }

    pub fn file_tree(&self) -> &[FileTreeNode] {
        &self.file_tree
    }

    pub fn toggle_expand(&mut self, path: &Path) {
        Self::toggle_node_expand(&mut self.file_tree, path);
    }

    fn toggle_node_expand(nodes: &mut [FileTreeNode], target: &Path) -> bool {
        for node in nodes.iter_mut() {
            if node.path == target {
                node.is_expanded = !node.is_expanded;
                return true;
            }
            if node.is_dir && target.starts_with(&node.path) {
                if Self::toggle_node_expand(&mut node.children, target) {
                    return true;
                }
            }
        }
        false
    }

    pub fn refresh_file_tree(&mut self) {
        self.file_tree = Self::scan_dir(&self.root_dir, 3);
    }

    fn scan_dir(dir: &Path, depth_limit: usize) -> Vec<FileTreeNode> {
        let mut nodes = Vec::new();
        if depth_limit == 0 || !dir.is_dir() {
            return nodes;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();

            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files/dirs like .git, .dsh
                if name.starts_with('.') {
                    continue;
                }

                if path.is_dir() {
                    let children = Self::scan_dir(&path, depth_limit - 1);
                    dirs.push(FileTreeNode {
                        name,
                        path,
                        is_dir: true,
                        children,
                        is_expanded: false,
                    });
                } else {
                    files.push(FileTreeNode {
                        name,
                        path,
                        is_dir: false,
                        children: Vec::new(),
                        is_expanded: false,
                    });
                }
            }

            dirs.sort_by(|a, b| a.name.cmp(&b.name));
            files.sort_by(|a, b| a.name.cmp(&b.name));

            nodes.extend(dirs);
            nodes.extend(files);
        }
        nodes
    }

    pub fn open_file(&mut self, title: String, path: Option<String>, content: &str) -> usize {
        // If file already opened, just activate it
        if let Some(ref p) = path {
            let found_idx = self.tabs.iter().position(|tab| tab.path.as_ref() == Some(p));
            if let Some(idx) = found_idx {
                self.select_tab(idx);
                return self.tabs[idx].id;
            }
        }

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
            is_dirty: false,
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

    pub fn active_tab_idx(&self) -> usize {
        self.active_tab_idx
    }

    pub fn tabs(&self) -> &[EditorTab] {
        &self.tabs
    }

    pub fn tabs_mut(&mut self) -> &mut [EditorTab] {
        &mut self.tabs
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
            } else {
                self.tabs[self.active_tab_idx].is_active = true;
            }
        }
    }
}
