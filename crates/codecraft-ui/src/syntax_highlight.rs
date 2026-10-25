use egui::text::LayoutJob;
use egui::{Color32, FontId, TextFormat};
use codecraft_core::syntax::{HighlightCategory, LanguageId, SyntaxEngine};
use crate::theme::Theme;

pub fn render_syntax_job(
    text: &str,
    lang: LanguageId,
    theme: &Theme,
    font_size: f32,
    wrap_width: f32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let syntax = SyntaxEngine::new(lang);
    let mono_font = FontId::monospace(font_size);

    let col_kw = Color32::from_rgb((theme.accent_rose[0]*255.0) as u8, (theme.accent_rose[1]*255.0) as u8, (theme.accent_rose[2]*255.0) as u8);
    let col_fn = Color32::from_rgb((theme.accent_blue[0]*255.0) as u8, (theme.accent_blue[1]*255.0) as u8, (theme.accent_blue[2]*255.0) as u8);
    let col_str = Color32::from_rgb((theme.accent_emerald[0]*255.0) as u8, (theme.accent_emerald[1]*255.0) as u8, (theme.accent_emerald[2]*255.0) as u8);
    let col_num = Color32::from_rgb((theme.accent_amber[0]*255.0) as u8, (theme.accent_amber[1]*255.0) as u8, (theme.accent_amber[2]*255.0) as u8);
    let col_comment = Color32::from_rgb((theme.fg_muted[0]*255.0) as u8, (theme.fg_muted[1]*255.0) as u8, (theme.fg_muted[2]*255.0) as u8);
    let col_normal = Color32::from_rgb((theme.fg_primary[0]*255.0) as u8, (theme.fg_primary[1]*255.0) as u8, (theme.fg_primary[2]*255.0) as u8);
    let col_punct = Color32::from_rgb((theme.border_subtle[0]*255.0*1.5).min(255.0) as u8, (theme.border_subtle[1]*255.0*1.5).min(255.0) as u8, (theme.border_subtle[2]*255.0*1.5).min(255.0) as u8);

    for (line_idx, line) in text.split('\n').enumerate() {
        if line_idx > 0 {
            job.append("\n", 0.0, TextFormat { font_id: mono_font.clone(), color: col_normal, ..Default::default() });
        }
        let tokens = syntax.tokenize_line(line);
        for (cat, val) in tokens {
            let color = match cat {
                HighlightCategory::Keyword => col_kw,
                HighlightCategory::Function => col_fn,
                HighlightCategory::Type => col_fn,
                HighlightCategory::String => col_str,
                HighlightCategory::Number => col_num,
                HighlightCategory::Comment => col_comment,
                HighlightCategory::Punctuation | HighlightCategory::Operator => col_punct,
                _ => col_normal,
            };
            job.append(&val, 0.0, TextFormat {
                font_id: mono_font.clone(),
                color,
                ..Default::default()
            });
        }
    }
    job
}
