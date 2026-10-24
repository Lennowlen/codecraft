use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutState {
    pub sidebar_visible: bool,
    pub sidebar_width: f32,
    pub terminal_visible: bool,
    pub terminal_height: f32,
    pub command_palette_open: bool,
    pub accessory_bar_visible: bool,
    pub memory_mb: f32,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            sidebar_visible: true,
            sidebar_width: 240.0,
            terminal_visible: true,
            terminal_height: 280.0,
            command_palette_open: false,
            accessory_bar_visible: true,
            memory_mb: 54.2,
        }
    }
}

impl LayoutState {
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
    }

    pub fn toggle_terminal(&mut self) {
        self.terminal_visible = !self.terminal_visible;
    }

    pub fn toggle_command_palette(&mut self) {
        self.command_palette_open = !self.command_palette_open;
    }
}
