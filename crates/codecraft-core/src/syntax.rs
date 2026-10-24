use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageId {
    Rust,
    Java,
    Python,
    C,
    Cpp,
    Go,
    JavaScript,
    TypeScript,
    Plain,
}

impl LanguageId {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => LanguageId::Rust,
            "java" => LanguageId::Java,
            "py" => LanguageId::Python,
            "c" | "h" => LanguageId::C,
            "cpp" | "hpp" | "cc" => LanguageId::Cpp,
            "go" => LanguageId::Go,
            "js" | "mjs" => LanguageId::JavaScript,
            "ts" => LanguageId::TypeScript,
            _ => LanguageId::Plain,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HighlightCategory {
    Keyword,
    Function,
    Type,
    String,
    Number,
    Comment,
    Operator,
    Punctuation,
    Variable,
    Normal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HighlightSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub category: HighlightCategory,
    pub color_rgba: [f32; 4],
}

pub struct SyntaxEngine {
    lang: LanguageId,
}

impl SyntaxEngine {
    pub fn new(lang: LanguageId) -> Self {
        Self { lang }
    }

    pub fn set_language(&mut self, lang: LanguageId) {
        self.lang = lang;
    }

    pub fn language(&self) -> LanguageId {
        self.lang
    }

    /// Fast token highlight generation for single line / viewport slice (fallback & low RAM)
    pub fn highlight_line(&self, line: &str, line_start_byte: usize) -> Vec<HighlightSpan> {
        let mut spans = Vec::new();
        if line.is_empty() {
            return spans;
        }

        let trimmed = line.trim_start();
        let leading_spaces = line.len() - trimmed.len();

        // C / Java / Rust comments
        if trimmed.starts_with("//") || trimmed.starts_with("#") {
            spans.push(HighlightSpan {
                start_byte: line_start_byte + leading_spaces,
                end_byte: line_start_byte + line.len(),
                category: HighlightCategory::Comment,
                color_rgba: [0.42, 0.45, 0.50, 1.0], // Dark zinc comment
            });
            return spans;
        }

        // Keywords detection
        let keywords: &[&str] = match self.lang {
            LanguageId::Rust => &[
                "fn", "pub", "struct", "enum", "impl", "use", "let", "mut", "match", "if", "else",
                "for", "while", "loop", "return", "true", "false", "mod", "trait", "async", "await"
            ],
            LanguageId::Java => &[
                "public", "private", "protected", "class", "interface", "void", "static", "final",
                "new", "return", "if", "else", "for", "while", "try", "catch", "import", "package"
            ],
            LanguageId::Python => &[
                "def", "class", "import", "from", "return", "if", "elif", "else", "for", "while",
                "try", "except", "with", "as", "True", "False", "None", "self"
            ],
            _ => &["fn", "class", "return", "if", "else", "for", "while"],
        };

        let mut byte_offset = 0;
        for word in line.split_whitespace() {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
            let is_kw = keywords.contains(&clean_word);
            let is_num = clean_word.parse::<f64>().is_ok();

            if is_kw {
                if let Some(pos) = line[byte_offset..].find(clean_word) {
                    let start = byte_offset + pos;
                    spans.push(HighlightSpan {
                        start_byte: line_start_byte + start,
                        end_byte: line_start_byte + start + clean_word.len(),
                        category: HighlightCategory::Keyword,
                        color_rgba: [0.77, 0.38, 0.88, 1.0], // Violet keyword
                    });
                }
            } else if is_num {
                if let Some(pos) = line[byte_offset..].find(clean_word) {
                    let start = byte_offset + pos;
                    spans.push(HighlightSpan {
                        start_byte: line_start_byte + start,
                        end_byte: line_start_byte + start + clean_word.len(),
                        category: HighlightCategory::Number,
                        color_rgba: [0.98, 0.75, 0.14, 1.0], // Amber number
                    });
                }
            }
            byte_offset += word.len() + 1;
            if byte_offset >= line.len() {
                break;
            }
        }

        spans
    }
}
