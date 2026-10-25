use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub bg_app: [f32; 4],
    pub bg_editor: [f32; 4],
    pub bg_sidebar: [f32; 4],
    pub bg_tabs: [f32; 4],
    pub bg_terminal: [f32; 4],
    pub bg_accessory_bar: [f32; 4],
    pub fg_primary: [f32; 4],
    pub fg_muted: [f32; 4],
    pub accent_blue: [f32; 4],
    pub accent_amber: [f32; 4],
    pub accent_emerald: [f32; 4],
    pub accent_rose: [f32; 4],
    pub border_subtle: [f32; 4],
    pub line_number: [f32; 4],
    pub line_number_active: [f32; 4],
    pub selection: [f32; 4],
}

impl Default for Theme {
    fn default() -> Self {
        Self::sublime_monokai()
    }
}

impl Theme {
    pub fn sublime_monokai() -> Self {
        Self {
            name: "Sublime Monokai Pro".to_string(),
            bg_app: [0.13, 0.13, 0.15, 1.0],           // #221F22
            bg_editor: [0.17, 0.16, 0.18, 1.0],        // #2D2A2E
            bg_sidebar: [0.13, 0.13, 0.15, 1.0],       // #221F22
            bg_tabs: [0.15, 0.14, 0.16, 1.0],          // #272428
            bg_terminal: [0.12, 0.11, 0.13, 1.0],      // #1F1C20
            bg_accessory_bar: [0.18, 0.17, 0.19, 1.0], // #2F2C30
            fg_primary: [0.97, 0.97, 0.96, 1.0],       // #FCFCFA
            fg_muted: [0.55, 0.53, 0.54, 1.0],         // #8F888B
            accent_blue: [0.47, 0.83, 0.89, 1.0],      // #78DCE8
            accent_amber: [1.00, 0.85, 0.40, 1.0],     // #FFD866
            accent_emerald: [0.65, 0.86, 0.46, 1.0],   // #A9DC76
            accent_rose: [1.00, 0.38, 0.53, 1.0],      // #FF6188
            border_subtle: [0.22, 0.20, 0.23, 1.0],    // #38343B
            line_number: [0.45, 0.43, 0.44, 1.0],      // #736E70
            line_number_active: [1.00, 0.85, 0.40, 1.0], // Gold
            selection: [0.25, 0.24, 0.28, 1.0],
        }
    }

    pub fn lapce_dark() -> Self {
        Self {
            name: "Lapce Dark Modern".to_string(),
            bg_app: [0.10, 0.10, 0.12, 1.0],           // #1B1B1E
            bg_editor: [0.12, 0.12, 0.14, 1.0],        // #1F1F24
            bg_sidebar: [0.09, 0.09, 0.11, 1.0],       // #17171C
            bg_tabs: [0.11, 0.11, 0.13, 1.0],          // #1C1C22
            bg_terminal: [0.08, 0.08, 0.10, 1.0],      // #14141A
            bg_accessory_bar: [0.14, 0.14, 0.16, 1.0], // #24242A
            fg_primary: [0.93, 0.93, 0.95, 1.0],       // #EEEEF2
            fg_muted: [0.50, 0.50, 0.56, 1.0],         // #80808F
            accent_blue: [0.35, 0.65, 0.95, 1.0],      // Lapce Blue
            accent_amber: [0.95, 0.72, 0.25, 1.0],     // Amber
            accent_emerald: [0.22, 0.80, 0.54, 1.0],   // Mint
            accent_rose: [0.92, 0.30, 0.40, 1.0],      // Crimson
            border_subtle: [0.18, 0.18, 0.22, 1.0],
            line_number: [0.40, 0.40, 0.46, 1.0],
            line_number_active: [0.35, 0.65, 0.95, 1.0],
            selection: [0.20, 0.26, 0.38, 1.0],
        }
    }

    pub fn artcraft_zinc() -> Self {
        Self {
            name: "ArtCraft Zinc (Emerald)".to_string(),
            bg_app: [0.07, 0.07, 0.08, 1.0],           // #121215
            bg_editor: [0.09, 0.09, 0.11, 1.0],        // #18181B
            bg_sidebar: [0.06, 0.06, 0.07, 1.0],       // #0F0F12
            bg_tabs: [0.08, 0.08, 0.10, 1.0],          // #14141A
            bg_terminal: [0.05, 0.05, 0.06, 1.0],      // #0D0D10
            bg_accessory_bar: [0.11, 0.11, 0.13, 1.0], // #1C1C20
            fg_primary: [0.94, 0.94, 0.95, 1.0],       // #F0F0F2
            fg_muted: [0.45, 0.45, 0.50, 1.0],         // #737380
            accent_blue: [0.22, 0.74, 0.97, 1.0],      // #38BDF8
            accent_amber: [0.98, 0.75, 0.14, 1.0],     // #FBBF24
            accent_emerald: [0.20, 0.83, 0.60, 1.0],   // #34D399
            accent_rose: [0.96, 0.25, 0.37, 1.0],      // #F43F5E
            border_subtle: [0.16, 0.16, 0.20, 1.0],    // #27272A
            line_number: [0.35, 0.35, 0.40, 1.0],
            line_number_active: [0.20, 0.83, 0.60, 1.0],
            selection: [0.15, 0.25, 0.22, 1.0],
        }
    }

    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn from_json(content: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(content)
    }
}
