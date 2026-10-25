use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityBarTab {
    Explorer,
    Search,
    SourceControl,
    Extensions,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BottomDockTab {
    Terminal,
    Output,
    Problems,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutState {
    pub active_activity_tab: ActivityBarTab,
    pub sidebar_visible: bool,
    pub sidebar_width: f32,
    pub active_dock_tab: BottomDockTab,
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
            active_activity_tab: ActivityBarTab::Explorer,
            sidebar_visible: true,
            sidebar_width: 250.0,
            active_dock_tab: BottomDockTab::Terminal,
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

    pub fn set_activity_tab(&mut self, tab: ActivityBarTab) {
        if self.active_activity_tab == tab && self.sidebar_visible {
            self.sidebar_visible = false;
        } else {
            self.active_activity_tab = tab;
            self.sidebar_visible = true;
        }
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
