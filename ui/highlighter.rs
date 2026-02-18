use super::theme::Theme;
use egui::{text::LayoutJob, Color32, FontId, TextFormat};

pub struct Token {
    pub text: String,
    pub color: Color32,
}

/// Stateless highlighter — cheaply re-created each frame.
#[derive(Clone, Copy)]
pub struct Highlighter {
    theme: Theme,
}

impl Highlighter {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Produce a colored `LayoutJob` from a full source string.
    /// Used as the `layouter` callback inside `TextEdit`.
    pub fn highlight_to_layout_job(&self, text: &str, font_id: FontId) -> LayoutJob {
        let mut job = LayoutJob::default();
        let mut lines = text.split('\n').peekable();
        while let Some(line) = lines.next() {
            for tok in self.tokenize_line(line) {
                job.append(
                    &tok.text,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: tok.color,
                        ..Default::default()
                    },
                );
            }
            // Restore the newline that split() consumed
            if lines.peek().is_some() {
                job.append(
                    "\n",
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: self.theme.text_default,
                        ..Default::default()
                    },
                );
            }
        }
        job
    }

    fn tokenize_line(&self, line: &str) -> Vec<Token> {
        let mut result = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        // Whole-line comment shortcut
        if line.trim_start().starts_with('#') {
            result.push(Token {
                text: line.to_string(),
                color: self.theme.comment,
            });
            return result;
        }

        while i < chars.len() {
            // Whitespace
            if chars[i].is_whitespace() {
                let start = i;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                result.push(Token {
                    text: chars[start..i].iter().collect(),
                    color: self.theme.text_default,
                });
                continue;
            }
            // Inline comment
            if chars[i] == '#' {
                result.push(Token {
                    text: chars[i..].iter().collect(),
                    color: self.theme.comment,
                });
                break;
            }
            // String literal
            if chars[i] == '"' {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if i < chars.len() {
                    i += 1;
                }
                result.push(Token {
                    text: chars[start..i].iter().collect(),
                    color: self.theme.string,
                });
                continue;
            }
            // Char literal
            if chars[i] == '\'' {
                let start = i;
                i += 1;
                if i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if i < chars.len() && chars[i] == '\'' {
                    i += 1;
                }
                result.push(Token {
                    text: chars[start..i].iter().collect(),
                    color: self.theme.char_lit,
                });
                continue;
            }
            // !keyword
            if chars[i] == '!' {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let color = if Self::is_keyword(&text[1..]) {
                    self.theme.keyword
                } else {
                    self.theme.operator
                };
                result.push(Token { text, color });
                continue;
            }
            // :type
            if chars[i] == ':' {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let color = if Self::is_type(&text[1..]) {
                    self.theme.type_name
                } else {
                    self.theme.operator
                };
                result.push(Token { text, color });
                continue;
            }
            // Operators / punctuation
            if Self::is_operator_char(chars[i]) {
                let start = i;
                while i < chars.len() && Self::is_operator_char(chars[i]) {
                    i += 1;
                }
                result.push(Token {
                    text: chars[start..i].iter().collect(),
                    color: self.theme.operator,
                });
                continue;
            }
            // Numbers
            if chars[i].is_numeric() {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_alphanumeric() || matches!(chars[i], '.' | 'x' | 'b' | 'o'))
                {
                    i += 1;
                }
                result.push(Token {
                    text: chars[start..i].iter().collect(),
                    color: self.theme.number,
                });
                continue;
            }
            // Identifiers
            if chars[i].is_alphabetic() || chars[i] == '_' {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let color = match text.as_str() {
                    "true" | "false" => self.theme.boolean,
                    "NULL" => self.theme.keyword,
                    _ => self.theme.identifier,
                };
                result.push(Token { text, color });
                continue;
            }
            // Fallback
            result.push(Token {
                text: chars[i].to_string(),
                color: self.theme.text_default,
            });
            i += 1;
        }
        result
    }

    fn is_keyword(s: &str) -> bool {
        matches!(
            s,
            "start"
                | "end"
                | "exit"
                | "if"
                | "else"
                | "for"
                | "while"
                | "func"
                | "return"
                | "struct"
                | "import"
                | "module"
        )
    }

    fn is_type(s: &str) -> bool {
        matches!(
            s,
            "int" | "float" | "char" | "boolean" | "array" | "list" | "struct"
        )
    }

    fn is_operator_char(c: char) -> bool {
        matches!(
            c,
            '+' | '-'
                | '*'
                | '/'
                | '%'
                | '&'
                | '|'
                | '~'
                | '^'
                | '='
                | '>'
                | '<'
                | '('
                | ')'
                | '{'
                | '}'
                | '['
                | ']'
                | '.'
                | ','
                | ';'
        )
    }
}
