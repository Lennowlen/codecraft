use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutState {
    pub sidebar_visible: bool,
    pub sidebar_width: f32,
    pub terminal_visible: bool,
    pub terminal_height: f32,
    pub command_palette_open: bool,
    pub accessory_bar_visible: bool,
    pub settings_open: bool,
    pub memory_mb: f32,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            sidebar_visible: true,
            sidebar_width: 250.0,
            terminal_visible: false,
            terminal_height: 240.0,
            command_palette_open: false,
            accessory_bar_visible: false, // Default off untuk tablet fisik / clean view (toggleable di settings)
            settings_open: false,
            memory_mb: 48.0,
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

    pub fn toggle_settings(&mut self) {
        self.settings_open = !self.settings_open;
    }
}
