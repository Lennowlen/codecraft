//! Lapce Codicons and UI Icons embedded as SVG and rendered crisp via egui_extras.

use egui::{Color32, Image, Rect, Ui, vec2};

pub static FILES_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\files.svg"#);
pub static SEARCH_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\search.svg"#);
pub static SCM_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\source-control.svg"#);
pub static EXTENSIONS_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\extensions.svg"#);
pub static SETTINGS_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\settings-gear.svg"#);
pub static RUN_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\debug-start.svg"#);
pub static CLOSE_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\close.svg"#);
pub static TERMINAL_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\terminal.svg"#);
pub static NEW_FILE_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\new-file.svg"#);
pub static NEW_FOLDER_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\new-folder.svg"#);
pub static REFRESH_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\refresh.svg"#);
pub static CHEVRON_RIGHT_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\chevron-right.svg"#);
pub static CHEVRON_DOWN_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\chevron-down.svg"#);
pub static FOLDER_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\folder.svg"#);
pub static FOLDER_OPENED_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\folder-opened.svg"#);
pub static FILE_CODE_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\file-code.svg"#);
pub static GIT_BRANCH_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\git-merge.svg"#);
pub static SPLIT_HORIZONTAL_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\split-horizontal.svg"#);
pub static REMOTE_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\lapce\lapce_remote.svg"#);
pub static ERROR_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\error.svg"#);
pub static WARNING_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\warning.svg"#);
pub static DISCARD_SVG: &[u8] = include_bytes!(r#"C:\Users\ThinkPad\Projects\artcraft-mobile\lapce-src\icons\codicons\discard.svg"#);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Codicon {
    Files,
    Search,
    SourceControl,
    Extensions,
    Settings,
    Run,
    Close,
    Terminal,
    NewFile,
    NewFolder,
    Refresh,
    ChevronRight,
    ChevronDown,
    Folder,
    FolderOpened,
    FileCode,
    GitBranch,
    SplitHorizontal,
    Remote,
    Error,
    Warning,
    Discard,
}

impl Codicon {
    pub fn bytes(&self) -> (&'static str, &'static [u8]) {
        match self {
            Codicon::Files => ("bytes://files.svg", FILES_SVG),
            Codicon::Search => ("bytes://search.svg", SEARCH_SVG),
            Codicon::SourceControl => ("bytes://source-control.svg", SCM_SVG),
            Codicon::Extensions => ("bytes://extensions.svg", EXTENSIONS_SVG),
            Codicon::Settings => ("bytes://settings-gear.svg", SETTINGS_SVG),
            Codicon::Run => ("bytes://debug-start.svg", RUN_SVG),
            Codicon::Close => ("bytes://close.svg", CLOSE_SVG),
            Codicon::Terminal => ("bytes://terminal.svg", TERMINAL_SVG),
            Codicon::NewFile => ("bytes://new-file.svg", NEW_FILE_SVG),
            Codicon::NewFolder => ("bytes://new-folder.svg", NEW_FOLDER_SVG),
            Codicon::Refresh => ("bytes://refresh.svg", REFRESH_SVG),
            Codicon::ChevronRight => ("bytes://chevron-right.svg", CHEVRON_RIGHT_SVG),
            Codicon::ChevronDown => ("bytes://chevron-down.svg", CHEVRON_DOWN_SVG),
            Codicon::Folder => ("bytes://folder.svg", FOLDER_SVG),
            Codicon::FolderOpened => ("bytes://folder-opened.svg", FOLDER_OPENED_SVG),
            Codicon::FileCode => ("bytes://file-code.svg", FILE_CODE_SVG),
            Codicon::GitBranch => ("bytes://git-merge.svg", GIT_BRANCH_SVG),
            Codicon::SplitHorizontal => ("bytes://split-horizontal.svg", SPLIT_HORIZONTAL_SVG),
            Codicon::Remote => ("bytes://lapce-remote.svg", REMOTE_SVG),
            Codicon::Error => ("bytes://error.svg", ERROR_SVG),
            Codicon::Warning => ("bytes://warning.svg", WARNING_SVG),
            Codicon::Discard => ("bytes://discard.svg", DISCARD_SVG),
        }
    }

    pub fn image(&self, size: f32, tint: Color32) -> Image<'static> {
        let (uri, data) = self.bytes();
        Image::from_bytes(uri, data)
            .fit_to_exact_size(vec2(size, size))
            .tint(tint)
    }

    pub fn paint(&self, ui: &Ui, rect: Rect, size: f32, tint: Color32) {
        let r = Rect::from_center_size(rect.center(), vec2(size, size));
        self.image(size, tint).paint_at(ui, r);
    }
}
