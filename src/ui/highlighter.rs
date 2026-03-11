use super::theme::Theme;
use egui::{text::LayoutJob, Color32, FontId, TextFormat};

pub struct Token {
    pub text: String,
    pub color: Color32,
}

#[derive(Clone, Copy)]
pub struct Highlighter {
    theme: Theme,
}

impl Highlighter {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn highlight_to_layout_job(&self, text: &str, font_id: FontId) -> LayoutJob {
        let mut job = LayoutJob::default();

        let mut in_block_comment = false;

        let mut lines = text.split('\n').peekable();
        while let Some(line) = lines.next() {
            let tokens = self.tokenize_line(line, &mut in_block_comment);
            for tok in tokens {
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

    fn tokenize_line(&self, line: &str, in_block_comment: &mut bool) -> Vec<Token> {
        let mut result = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        if *in_block_comment {
            let mut seg = String::new();
            while i < chars.len() {
                if i + 2 < chars.len()
                    && chars[i] == '#'
                    && chars[i + 1] == '#'
                    && chars[i + 2] == '#'
                {
                    seg.push_str("###");
                    i += 3;
                    *in_block_comment = false;
                    break;
                } else {
                    seg.push(chars[i]);
                    i += 1;
                }
            }
            if !seg.is_empty() {
                result.push(Token {
                    text: seg,
                    color: self.theme.comment,
                });
            }
            if *in_block_comment {
                return result;
            }
        }

        {
            let rest = &line[line
                .char_indices()
                .nth(i)
                .map(|(b, _)| b)
                .unwrap_or(line.len())..];
            let trimmed = rest.trim_start();
            if trimmed.starts_with("###") {
            } else if trimmed.starts_with('#') && i == chars.iter().take(i).count() {
                if result.is_empty() {
                    result.push(Token {
                        text: line.to_string(),
                        color: self.theme.comment,
                    });
                    return result;
                }
            }
        }

        let mut next_ident_is_fn = false;

        while i < chars.len() {
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

            if chars[i] == '#' && i + 2 < chars.len() && chars[i + 1] == '#' && chars[i + 2] == '#'
            {
                let mut seg = String::from("###");
                i += 3;
                *in_block_comment = true;

                while i < chars.len() {
                    if i + 2 < chars.len()
                        && chars[i] == '#'
                        && chars[i + 1] == '#'
                        && chars[i + 2] == '#'
                    {
                        seg.push_str("###");
                        i += 3;
                        *in_block_comment = false;
                        break;
                    } else {
                        seg.push(chars[i]);
                        i += 1;
                    }
                }

                result.push(Token {
                    text: seg,
                    color: self.theme.comment,
                });
                continue;
            }

            if chars[i] == '#' {
                result.push(Token {
                    text: chars[i..].iter().collect(),
                    color: self.theme.comment,
                });
                break;
            }

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

            if chars[i] == '!' {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let word = &text[1..];
                let color = if Self::is_keyword(word) {
                    if word == "func" {
                        next_ident_is_fn = true;
                    }
                    self.theme.keyword
                } else {
                    self.theme.operator
                };
                result.push(Token { text, color });
                continue;
            }

            if chars[i] == ':' && i + 1 < chars.len() && chars[i + 1] == ':' {
                result.push(Token {
                    text: "::".to_string(),
                    color: self.theme.operator,
                });
                i += 2;
                continue;
            }

            if chars[i] == ':' {
                i += 1;
                let mut type_word = String::new();
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    type_word.push(chars[i]);
                    i += 1;
                }

                if Self::is_type(&type_word) {
                    result.push(Token {
                        text: format!(":{}", type_word),
                        color: self.theme.type_name,
                    });

                    if type_word == "struct" && i < chars.len() && chars[i] == '<' {
                        result.push(Token {
                            text: "<".to_string(),
                            color: self.theme.angle_bracket,
                        });
                        i += 1;

                        let name_start = i;
                        while i < chars.len() && chars[i] != '>' {
                            i += 1;
                        }
                        let struct_name: String = chars[name_start..i].iter().collect();
                        if !struct_name.is_empty() {
                            result.push(Token {
                                text: struct_name,
                                color: self.theme.struct_name,
                            });
                        }

                        if i < chars.len() && chars[i] == '>' {
                            result.push(Token {
                                text: ">".to_string(),
                                color: self.theme.angle_bracket,
                            });
                            i += 1;
                        }
                    }
                } else {
                    result.push(Token {
                        text: format!(":{}", type_word),
                        color: self.theme.operator,
                    });
                }
                continue;
            }

            if chars[i] == '-' && i + 1 < chars.len() && chars[i + 1] == '>' {
                result.push(Token {
                    text: "->".to_string(),
                    color: self.theme.operator,
                });
                i += 2;
                continue;
            }

            if Self::is_bracket_char(chars[i]) {
                result.push(Token {
                    text: chars[i].to_string(),
                    color: self.theme.bracket,
                });
                i += 1;
                continue;
            }

            if Self::is_operator_char(chars[i]) && i + 1 < chars.len() {
                let two: String = chars[i..i + 2].iter().collect();
                if matches!(
                    two.as_str(),
                    ">=" | "<="
                        | "=="
                        | "~="
                        | "+="
                        | "-="
                        | "*="
                        | "/="
                        | "%="
                        | "&="
                        | "|="
                        | "^="
                ) {
                    result.push(Token {
                        text: two,
                        color: self.theme.operator,
                    });
                    i += 2;
                    continue;
                }
            }

            if Self::is_operator_char(chars[i]) {
                result.push(Token {
                    text: chars[i].to_string(),
                    color: self.theme.operator,
                });
                i += 1;
                continue;
            }

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

            if chars[i].is_alphabetic() || chars[i] == '_' {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();

                let mut j = i;
                while j < chars.len() && chars[j] == ' ' {
                    j += 1;
                }
                let followed_by_paren = j < chars.len() && chars[j] == '(';

                let color = match text.as_str() {
                    "true" | "false" => self.theme.boolean,
                    "NULL" => self.theme.keyword,
                    _ if next_ident_is_fn => self.theme.fn_name,
                    _ if followed_by_paren => self.theme.fn_name,
                    _ => self.theme.identifier,
                };

                if next_ident_is_fn {
                    next_ident_is_fn = false;
                }

                result.push(Token { text, color });
                continue;
            }

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
                | "elif"
                | "else"
                | "for"
                | "while"
                | "func"
                | "return"
                | "struct"
                | "import"
                | "module"
                | "break"
                | "continue"
                | "and"
                | "or"
                | "not"
                | "null"
        )
    }

    fn is_type(s: &str) -> bool {
        matches!(
            s,
            "int" | "float" | "char" | "boolean" | "array" | "list" | "struct" | "void"
        )
    }

    fn is_bracket_char(c: char) -> bool {
        matches!(c, '(' | ')' | '{' | '}' | '[' | ']')
    }

    fn is_operator_char(c: char) -> bool {
        matches!(
            c,
            '+' | '-' | '*' | '/' | '%' | '&' | '|' | '~' | '^' | '=' | '>' | '<' | '.' | ',' | ';'
        )
    }
}
