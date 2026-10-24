pub struct Theme {
    pub bg_app: [f32; 4],
    pub bg_editor: [f32; 4],
    pub bg_sidebar: [f32; 4],
    pub bg_accessory_bar: [f32; 4],
    pub fg_primary: [f32; 4],
    pub fg_muted: [f32; 4],
    pub accent_blue: [f32; 4],
    pub accent_amber: [f32; 4],
    pub accent_emerald: [f32; 4],
    pub accent_rose: [f32; 4],
    pub border_subtle: [f32; 4],
}

impl Default for Theme {
    fn default() -> Self {
        // VS Code Dark Modern / ArtCraft Dark Zinc Palette
        Self {
            bg_app: [0.07, 0.07, 0.08, 1.0],           // #121215
            bg_editor: [0.09, 0.09, 0.11, 1.0],        // #18181B
            bg_sidebar: [0.06, 0.06, 0.07, 1.0],       // #0F0F12
            bg_accessory_bar: [0.11, 0.11, 0.13, 1.0], // #1C1C20
            fg_primary: [0.94, 0.94, 0.95, 1.0],       // #F0F0F2
            fg_muted: [0.45, 0.45, 0.50, 1.0],         // #737380
            accent_blue: [0.22, 0.74, 0.97, 1.0],      // #38BDF8
            accent_amber: [0.98, 0.75, 0.14, 1.0],     // #FBBF24
            accent_emerald: [0.20, 0.83, 0.60, 1.0],   // #34D399
            accent_rose: [0.96, 0.25, 0.37, 1.0],      // #F43F5E
            border_subtle: [0.16, 0.16, 0.20, 1.0],    // #27272A
        }
    }
}
