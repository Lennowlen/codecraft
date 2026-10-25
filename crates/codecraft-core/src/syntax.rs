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

#[derive(Debug, Clone)]
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

    /// Fast, robust tokenizer that decomposes text into styled syntax sections.
    pub fn tokenize_line(&self, line: &str) -> Vec<(HighlightCategory, String)> {
        let mut tokens = Vec::new();
        if line.is_empty() {
            return tokens;
        }

        let trimmed = line.trim_start();
        let leading_spaces = line.len() - trimmed.len();
        if leading_spaces > 0 {
            tokens.push((HighlightCategory::Normal, line[..leading_spaces].to_string()));
        }

        let content = &line[leading_spaces..];
        // Comments
        if content.starts_with("//") || content.starts_with("#") {
            tokens.push((HighlightCategory::Comment, content.to_string()));
            return tokens;
        }

        let keywords: &[&str] = match self.lang {
            LanguageId::Rust => &[
                "fn", "pub", "struct", "enum", "impl", "use", "let", "mut", "match", "if", "else",
                "for", "while", "loop", "return", "true", "false", "mod", "trait", "async", "await",
                "crate", "super", "self", "Self", "where", "as", "const", "static", "type", "ref"
            ],
            LanguageId::Java => &[
                "public", "private", "protected", "class", "interface", "void", "static", "final",
                "new", "return", "if", "else", "for", "while", "try", "catch", "import", "package",
                "extends", "implements", "this", "super", "null", "true", "false"
            ],
            LanguageId::Python => &[
                "def", "class", "import", "from", "return", "if", "elif", "else", "for", "while",
                "try", "except", "with", "as", "True", "False", "None", "self", "pass", "lambda",
                "yield", "in", "is", "not", "and", "or"
            ],
            LanguageId::C | LanguageId::Cpp => &[
                "int", "char", "float", "double", "void", "struct", "class", "return", "if", "else",
                "for", "while", "switch", "case", "break", "continue", "include", "typedef", "static"
            ],
            _ => &["fn", "class", "return", "if", "else", "for", "while", "true", "false"],
        };

        let chars: Vec<char> = content.chars().collect();
        let mut i = 0;
        let len = chars.len();

        while i < len {
            let c = chars[i];

            // String literal
            if c == '"' || c == '\'' {
                let quote = c;
                let start = i;
                i += 1;
                while i < len {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 2;
                    } else if chars[i] == quote {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
                let s: String = chars[start..i].iter().collect();
                tokens.push((HighlightCategory::String, s));
                continue;
            }

            // Word / Identifier / Number
            if c.is_alphanumeric() || c == '_' {
                let start = i;
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                if keywords.contains(&word.as_str()) {
                    tokens.push((HighlightCategory::Keyword, word));
                } else if word.chars().all(|ch| ch.is_ascii_digit()) {
                    tokens.push((HighlightCategory::Number, word));
                } else if i < len && chars[i] == '(' {
                    tokens.push((HighlightCategory::Function, word));
                } else if word.starts_with(|ch: char| ch.is_uppercase()) {
                    tokens.push((HighlightCategory::Type, word));
                } else {
                    tokens.push((HighlightCategory::Normal, word));
                }
                continue;
            }

            // Punctuation / Operator / Whitespace
            let start = i;
            while i < len && !chars[i].is_alphanumeric() && chars[i] != '_' && chars[i] != '"' && chars[i] != '\'' {
                if chars[i] == '/' && i + 1 < len && chars[i + 1] == '/' {
                    break;
                }
                i += 1;
            }
            let sym: String = chars[start..i].iter().collect();
            tokens.push((HighlightCategory::Punctuation, sym));
        }

        tokens
    }
}
