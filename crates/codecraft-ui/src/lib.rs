pub mod theme;
pub mod layout;
pub mod icons;
pub mod syntax_highlight;

pub use theme::Theme;
pub use layout::{LayoutState, ActivityBarTab, BottomDockTab};
pub use icons::Codicon;
pub use syntax_highlight::render_syntax_job;
