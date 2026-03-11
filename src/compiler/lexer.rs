#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Start,
    End,

    Exit,
    If,
    Elif,
    Else,
    For,
    While,
    Func,
    Return,
    Struct,
    Import,
    Module,
    Break,
    Continue,

    And,
    Or,
    Not,

    TypeInt,
    TypeFloat,
    TypeChar,
    TypeBoolean,
    TypeArray,
    TypeList,
    TypeStruct,
    TypeVoid,

    SIntLit(i64),
    FloatLit(f64),
    CharLit(char),
    StringLit(String),
    BoolLit(bool),
    Null,

    Identifier(String),

    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    Ampersand,
    Pipe,
    Caret,
    Tilde,

    Equals,
    PlusEquals,
    MinusEquals,
    StarEquals,
    SlashEquals,
    PercentEquals,

    AmpersandEquals,
    PipeEquals,
    CaretEquals,

    Greater,
    Less,
    GreaterEquals,
    LessEquals,
    EqualsEquals,
    TildeEquals,

    Arrow,
    Dot,
    Comma,
    ColonColon,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    EndL,

    NoMatch,

    ModuleStart(String),
    ModuleEnd(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
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

fn is_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

fn parse_number_literal(s: &str) -> TokenType {
    if s.starts_with("0b") {
        if let Ok(val) = i64::from_str_radix(&s[2..], 2) {
            return TokenType::SIntLit(val);
        }
    } else if s.starts_with("0x") {
        if let Ok(val) = i64::from_str_radix(&s[2..], 16) {
            return TokenType::SIntLit(val);
        }
    } else if s.starts_with("0o") {
        if let Ok(val) = i64::from_str_radix(&s[2..], 8) {
            return TokenType::SIntLit(val);
        }
    } else if s.starts_with("0d") {
        if let Ok(val) = s[2..].parse::<i64>() {
            return TokenType::SIntLit(val);
        }
    } else if s.contains('.') || s.contains('e') || s.contains('E') {
        if let Ok(val) = s.parse::<f64>() {
            return TokenType::FloatLit(val);
        }
    } else if let Ok(val) = s.parse::<i64>() {
        return TokenType::SIntLit(val);
    }
    TokenType::NoMatch
}

fn operator_map(s: &str) -> TokenType {
    match s {
        ";" => TokenType::EndL,
        "+" => TokenType::Plus,
        "-" => TokenType::Minus,
        "*" => TokenType::Star,
        "/" => TokenType::Slash,
        "%" => TokenType::Percent,
        "&" => TokenType::Ampersand,
        "|" => TokenType::Pipe,
        "^" => TokenType::Caret,
        "~" => TokenType::Tilde,
        "=" => TokenType::Equals,
        "+=" => TokenType::PlusEquals,
        "-=" => TokenType::MinusEquals,
        "*=" => TokenType::StarEquals,
        "/=" => TokenType::SlashEquals,
        "%=" => TokenType::PercentEquals,
        "&=" => TokenType::AmpersandEquals,
        "|=" => TokenType::PipeEquals,
        "^=" => TokenType::CaretEquals,
        ">" => TokenType::Greater,
        "<" => TokenType::Less,
        ">=" => TokenType::GreaterEquals,
        "<=" => TokenType::LessEquals,
        "==" => TokenType::EqualsEquals,
        "~=" => TokenType::TildeEquals,
        "(" => TokenType::LParen,
        ")" => TokenType::RParen,
        "{" => TokenType::LBrace,
        "}" => TokenType::RBrace,
        "[" => TokenType::LBracket,
        "]" => TokenType::RBracket,
        "->" => TokenType::Arrow,
        "." => TokenType::Dot,
        "," => TokenType::Comma,
        "::" => TokenType::ColonColon,
        _ => TokenType::NoMatch,
    }
}

fn keyword_map(s: &str) -> TokenType {
    match s {
        "start" => TokenType::Start,
        "end" => TokenType::End,
        "exit" => TokenType::Exit,
        "if" => TokenType::If,
        "elif" => TokenType::Elif,
        "else" => TokenType::Else,
        "for" => TokenType::For,
        "while" => TokenType::While,
        "func" => TokenType::Func,
        "return" => TokenType::Return,
        "struct" => TokenType::Struct,
        "import" => TokenType::Import,
        "module" => TokenType::Module,
        "break" => TokenType::Break,
        "continue" => TokenType::Continue,
        "and" => TokenType::And,
        "or" => TokenType::Or,
        "not" => TokenType::Not,
        "null" => TokenType::Null,
        _ => TokenType::NoMatch,
    }
}

fn type_map(s: &str) -> TokenType {
    match s {
        "int" => TokenType::TypeInt,
        "float" => TokenType::TypeFloat,
        "char" => TokenType::TypeChar,
        "boolean" => TokenType::TypeBoolean,
        "array" => TokenType::TypeArray,
        "list" => TokenType::TypeList,
        "struct" => TokenType::TypeStruct,
        "void" => TokenType::TypeVoid,
        _ => TokenType::NoMatch,
    }
}

fn handle_token(buffer: &str) -> Token {
    match buffer {
        "true" => {
            return Token {
                token_type: TokenType::BoolLit(true),
            }
        }
        "false" => {
            return Token {
                token_type: TokenType::BoolLit(false),
            }
        }
        "!null" => {
            return Token {
                token_type: TokenType::Null,
            }
        }
        _ => {}
    }

    let op = operator_map(buffer);
    if !matches!(op, TokenType::NoMatch) {
        return Token { token_type: op };
    }

    let num = parse_number_literal(buffer);
    if !matches!(num, TokenType::NoMatch) {
        return Token { token_type: num };
    }

    if is_identifier(buffer) {
        return Token {
            token_type: TokenType::Identifier(buffer.to_string()),
        };
    }

    Token {
        token_type: TokenType::NoMatch,
    }
}

fn parse_module_marker(chars: &[char], start_index: usize) -> Option<(TokenType, usize)> {
    let mut i = start_index;
    if i >= chars.len() || chars[i] != '$' {
        return None;
    }
    i += 1;

    let mut marker = String::new();
    while i < chars.len() && chars[i] != '$' {
        marker.push(chars[i]);
        i += 1;
    }
    if i >= chars.len() {
        return None;
    }
    i += 1;

    if let Some(name) = marker.strip_prefix("MODULE_START:") {
        return Some((TokenType::ModuleStart(name.to_string()), i));
    }
    if let Some(name) = marker.strip_prefix("MODULE_END:") {
        return Some((TokenType::ModuleEnd(name.to_string()), i));
    }
    None
}

pub fn tokenize(program: &str) -> Vec<Token> {
    let chars: Vec<char> = program.chars().collect();
    let mut tokens: Vec<Token> = Vec::new();
    let mut index: usize = 0;

    macro_rules! peek {
        ($offset:expr) => {
            chars.get(index + $offset).copied()
        };
    }

    while index < chars.len() {
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        if index >= chars.len() {
            break;
        }

        if chars[index] == '$' {
            if let Some((tt, new_index)) = parse_module_marker(&chars, index) {
                tokens.push(Token { token_type: tt });
                index = new_index;
                continue;
            }
        }

        if chars[index] == ':' && peek!(1) == Some(':') {
            tokens.push(Token {
                token_type: TokenType::ColonColon,
            });
            index += 2;
            continue;
        }

        if chars[index] == '!' {
            index += 1;
            let mut buffer = String::new();
            while index < chars.len()
                && !chars[index].is_whitespace()
                && !is_operator_char(chars[index])
            {
                buffer.push(chars[index]);
                index += 1;
            }
            let result = keyword_map(&buffer);
            tokens.push(Token {
                token_type: if matches!(result, TokenType::NoMatch) {
                    TokenType::NoMatch
                } else {
                    result
                },
            });
            continue;
        }

        if chars[index] == ':' {
            index += 1;
            let mut buffer = String::new();
            while index < chars.len()
                && !chars[index].is_whitespace()
                && !is_operator_char(chars[index])
                && chars[index] != '<'
            {
                buffer.push(chars[index]);
                index += 1;
            }
            let result = type_map(&buffer);
            tokens.push(Token {
                token_type: if matches!(result, TokenType::NoMatch) {
                    TokenType::NoMatch
                } else {
                    result
                },
            });
            continue;
        }

        if chars[index] == '"' {
            index += 1;
            let mut buf = String::new();
            while index < chars.len() && chars[index] != '"' {
                if chars[index] == '\\' && index + 1 < chars.len() {
                    index += 1;
                    buf.push(match chars[index] {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        c => c,
                    });
                } else {
                    buf.push(chars[index]);
                }
                index += 1;
            }
            if index < chars.len() {
                index += 1;
            }
            tokens.push(Token {
                token_type: TokenType::StringLit(buf),
            });
            continue;
        }

        if chars[index] == '\'' {
            index += 1;
            let char_val = if index < chars.len() {
                if chars[index] == '\\' && index + 1 < chars.len() {
                    index += 1;
                    let c = match chars[index] {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '\'' => '\'',
                        c => c,
                    };
                    index += 1;
                    c
                } else {
                    let c = chars[index];
                    index += 1;
                    c
                }
            } else {
                '\0'
            };
            if index < chars.len() && chars[index] == '\'' {
                index += 1;
            }
            tokens.push(Token {
                token_type: TokenType::CharLit(char_val),
            });
            continue;
        }

        if is_operator_char(chars[index]) {
            if index + 1 < chars.len() {
                let two = format!("{}{}", chars[index], chars[index + 1]);
                let result = operator_map(&two);
                if !matches!(result, TokenType::NoMatch) {
                    tokens.push(Token { token_type: result });
                    index += 2;
                    continue;
                }
            }
            let one = chars[index].to_string();
            let result = operator_map(&one);
            if !matches!(result, TokenType::NoMatch) {
                tokens.push(Token { token_type: result });
            }
            index += 1;
            continue;
        }

        let mut buffer = String::new();
        while index < chars.len() {
            let c = chars[index];
            if c.is_whitespace() || c == '!' || c == ':' || c == '$' {
                break;
            }

            if c == '.' {
                let buf_is_numeric = buffer
                    .chars()
                    .next()
                    .map(|ch| ch.is_ascii_digit())
                    .unwrap_or(false);
                let next_is_digit = chars
                    .get(index + 1)
                    .map(|&nc| nc.is_ascii_digit())
                    .unwrap_or(false);
                if buf_is_numeric && next_is_digit {
                    buffer.push(c);
                    index += 1;
                    continue;
                } else {
                    break;
                }
            }
            if is_operator_char(c) {
                break;
            }
            buffer.push(c);
            index += 1;
        }
        if !buffer.is_empty() {
            tokens.push(handle_token(&buffer));
        }
    }

    tokens
}
