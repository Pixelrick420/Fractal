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

    FileMap(String, usize),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub col: usize,
    pub file: String,
}

fn offset_to_line_col(src: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, c) in src.char_indices() {
        if i == offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn get_source_line(src: &str, line: usize) -> &str {
    src.lines().nth(line - 1).unwrap_or("")
}

fn emit_error(
    src: &str,
    source_file: &str,
    offset: usize,
    span_len: usize,
    code: &str,
    title: &str,
    label: &str,
    hint: &str,
) {
    let (line, col) = offset_to_line_col(src, offset);
    let src_line = get_source_line(src, line);
    let line_str = line.to_string();
    let pad = " ".repeat(line_str.len());
    let underline_len = span_len.max(1);
    let underline = "^".repeat(underline_len);
    let caret_pad = " ".repeat(col.saturating_sub(1));

    let display_file = std::path::Path::new(source_file)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(source_file);

    eprintln!(
        "\x1b[1;31merror[{code}]\x1b[0m\x1b[1m: {title}\x1b[0m",
        code = code,
        title = title
    );
    eprintln!(
        " \x1b[1;34m-->\x1b[0m {file}:{line}:{col}",
        file = display_file,
        line = line,
        col = col
    );
    eprintln!(" \x1b[1;34m{pad} |\x1b[0m");
    eprintln!(
        " \x1b[1;34m{line_str} |\x1b[0m {src_line}",
        line_str = line_str,
        src_line = src_line
    );
    eprintln!(" \x1b[1;34m{pad} |\x1b[0m \x1b[1;31m{caret_pad}{underline} {label}\x1b[0m");
    if !hint.is_empty() {
        eprintln!(" \x1b[1;34m{pad} =\x1b[0m \x1b[1;32mhint\x1b[0m: {hint}");
    }
    eprintln!();
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

fn closest_keyword(s: &str) -> Option<&'static str> {
    const KEYWORDS: &[&str] = &[
        "start", "end", "exit", "if", "elif", "else", "for", "while", "func", "return", "struct",
        "import", "module", "break", "continue", "and", "or", "not", "null",
    ];

    KEYWORDS.iter().copied().find(|kw| {
        let a: Vec<char> = s.chars().collect();
        let b: Vec<char> = kw.chars().collect();
        if a.len().abs_diff(b.len()) > 1 {
            return false;
        }
        let diffs = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
        diffs <= 1 + a.len().abs_diff(b.len())
    })
}

fn closest_type(s: &str) -> Option<&'static str> {
    const TYPES: &[&str] = &[
        "int", "float", "char", "boolean", "array", "list", "struct", "void",
    ];
    TYPES.iter().copied().find(|t| {
        let a: Vec<char> = s.chars().collect();
        let b: Vec<char> = t.chars().collect();
        if a.len().abs_diff(b.len()) > 1 {
            return false;
        }
        let diffs = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
        diffs <= 1 + a.len().abs_diff(b.len())
    })
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

fn classify_buffer(buffer: &str) -> TokenType {
    match buffer {
        "true" => return TokenType::BoolLit(true),
        "false" => return TokenType::BoolLit(false),
        _ => {}
    }

    let num = parse_number_literal(buffer);
    if !matches!(num, TokenType::NoMatch) {
        return num;
    }

    if is_identifier(buffer) {
        return TokenType::Identifier(buffer.to_string());
    }

    TokenType::NoMatch
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
    if let Some(rest) = marker.strip_prefix("SRCMAP:") {
        if let Some(colon) = rest.rfind(':') {
            let file = &rest[..colon];
            let line_str = &rest[colon + 1..];
            if let Ok(line) = line_str.parse::<usize>() {
                return Some((TokenType::FileMap(file.to_string(), line), i));
            }
        }
    }
    None
}

pub fn tokenize_with_source(program: &str, source_file: &str) -> Vec<Token> {
    let chars: Vec<char> = program.chars().collect();
    let mut tokens: Vec<Token> = Vec::new();
    let mut index: usize = 0;
    let mut had_error = false;

    let mut map_file = source_file.to_string();
    let mut map_line: usize = 1;
    let mut map_col: usize = 1;

    let char_offsets: Vec<usize> = {
        let mut offs = Vec::with_capacity(chars.len() + 1);
        let mut byte = 0usize;
        for c in &chars {
            offs.push(byte);
            byte += c.len_utf8();
        }
        offs.push(byte);
        offs
    };

    macro_rules! byte_off {
        ($ci:expr) => {
            char_offsets.get($ci).copied().unwrap_or(program.len())
        };
    }

    macro_rules! peek {
        ($offset:expr) => {
            chars.get(index + $offset).copied()
        };
    }

    while index < chars.len() {
        while index < chars.len() && chars[index].is_whitespace() {
            if chars[index] == '\n' {
                map_line += 1;
                map_col = 1;
            } else {
                map_col += 1;
            }
            index += 1;
        }
        if index >= chars.len() {
            break;
        }

        let token_start = index;
        let tok_line = map_line;
        let tok_col = map_col;

        if chars[index] == '$' {
            if let Some((tt, new_index)) = parse_module_marker(&chars, index) {
                for i in index..new_index {
                    if chars[i] == '\n' {
                        map_line += 1;
                        map_col = 1;
                    } else {
                        map_col += 1;
                    }
                }
                index = new_index;
                if let TokenType::FileMap(ref file, line) = tt {
                    map_file = file.clone();
                    map_line = line;
                    map_col = 1;
                } else {
                    tokens.push(Token {
                        token_type: tt,
                        line: tok_line,
                        col: tok_col,
                        file: map_file.clone(),
                    });
                }
                continue;
            }
            emit_error(
                program,
                &map_file,
                byte_off!(token_start),
                1,
                "E001",
                "unexpected character `$`",
                "not a valid token",
                "the `$` character is reserved for internal module markers; remove it",
            );
            had_error = true;
            map_col += 1;
            index += 1;
            continue;
        }

        if chars[index] == ':' && peek!(1) == Some(':') {
            tokens.push(Token {
                token_type: TokenType::ColonColon,
                line: tok_line,
                col: tok_col,
                file: map_file.clone(),
            });
            map_col += 2;
            index += 2;
            continue;
        }

        if chars[index] == '!' {
            let bang_pos = token_start;
            map_col += 1;
            index += 1;
            let mut buffer = String::new();
            while index < chars.len()
                && !chars[index].is_whitespace()
                && !is_operator_char(chars[index])
            {
                buffer.push(chars[index]);
                map_col += 1;
                index += 1;
            }
            if buffer.is_empty() {
                emit_error(
                    program,
                    &map_file,
                    byte_off!(bang_pos),
                    1,
                    "E002",
                    "bare `!` with no keyword",
                    "expected a keyword after `!`",
                    "valid keywords: `!if`, `!else`, `!elif`, `!for`, `!while`, `!func`, \
                     `!return`, `!break`, `!continue`, `!import`, `!start`, `!end`, `!exit`",
                );
                had_error = true;
                continue;
            }
            let result = keyword_map(&buffer);
            if matches!(result, TokenType::NoMatch) {
                let hint = if buffer == "true" || buffer == "false" {
                    format!(
                        "`!{buffer}` is not valid — boolean literals do not use the `!` prefix; \
                         write `{buffer}` directly"
                    )
                } else if let Some(close) = closest_keyword(&buffer) {
                    format!("unknown keyword `!{buffer}` — did you mean `!{close}`?")
                } else {
                    format!(
                        "unknown keyword `!{buffer}`; valid keywords: \
                         if, elif, else, for, while, func, return, break, continue, \
                         import, start, end, exit, struct, module"
                    )
                };
                emit_error(
                    program,
                    &map_file,
                    byte_off!(bang_pos),
                    buffer.len() + 1,
                    "E003",
                    &format!("unknown keyword `!{}`", buffer),
                    "not a recognised keyword",
                    &hint,
                );
                had_error = true;
            } else {
                tokens.push(Token {
                    token_type: result,
                    line: tok_line,
                    col: tok_col,
                    file: map_file.clone(),
                });
            }
            continue;
        }

        if chars[index] == ':' {
            let colon_pos = token_start;
            map_col += 1;
            index += 1;
            let mut buffer = String::new();
            while index < chars.len()
                && !chars[index].is_whitespace()
                && !is_operator_char(chars[index])
                && chars[index] != '<'
            {
                buffer.push(chars[index]);
                map_col += 1;
                index += 1;
            }
            if buffer.is_empty() {
                emit_error(
                    program,
                    &map_file,
                    byte_off!(colon_pos),
                    1,
                    "E004",
                    "bare `:` with no type name",
                    "expected a type name after `:`",
                    "types are written as `:int`, `:float`, `:char`, `:boolean`, \
                     `:array<T,N>`, `:list<T>`, `:struct<n>`, `:void`; for field access use `::`",
                );
                had_error = true;
                continue;
            }
            let result = type_map(&buffer);
            if matches!(result, TokenType::NoMatch) {
                let hint = if let Some(close) = closest_type(&buffer) {
                    format!("unknown type `:{buffer}` — did you mean `:{close}`?")
                } else {
                    format!("unknown type `:{buffer}`; valid primitive types: \
                         int, float, char, boolean, void; generic: array<T,N>, list<T>, struct<Name>")
                };
                emit_error(
                    program,
                    &map_file,
                    byte_off!(colon_pos),
                    buffer.len() + 1,
                    "E005",
                    &format!("unknown type `:{}`", buffer),
                    "not a recognised type",
                    &hint,
                );
                had_error = true;
            } else {
                tokens.push(Token {
                    token_type: result,
                    line: tok_line,
                    col: tok_col,
                    file: map_file.clone(),
                });
            }
            continue;
        }

        if chars[index] == '"' {
            let str_start = token_start;
            map_col += 1;
            index += 1;
            let mut buf = String::new();
            let mut closed = false;
            while index < chars.len() {
                if chars[index] == '"' {
                    closed = true;
                    map_col += 1;
                    index += 1;
                    break;
                }
                if chars[index] == '\n' {
                    map_line += 1;
                    map_col = 1;
                    index += 1;
                    break;
                }
                if chars[index] == '\\' {
                    if index + 1 >= chars.len() {
                        break;
                    }
                    map_col += 1;
                    index += 1;
                    let escaped = match chars[index] {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '0' => '\0',
                        c => {
                            emit_error(
                                program,
                                &map_file,
                                byte_off!(index - 1),
                                2,
                                "E006",
                                &format!("unknown escape sequence `\\{c}`"),
                                "invalid escape",
                                "valid escapes: `\\n` `\\t` `\\r` `\\\\` `\\\"` `\\0`",
                            );
                            had_error = true;
                            c
                        }
                    };
                    buf.push(escaped);
                    map_col += 1;
                    index += 1;
                    continue;
                }
                buf.push(chars[index]);
                map_col += 1;
                index += 1;
            }
            if !closed {
                emit_error(program, &map_file, byte_off!(str_start), 1, "E007",
                    "unterminated string literal", "string starts here, never closed",
                    "add a closing `\"` at the end of the string; strings cannot span multiple lines");
                had_error = true;
            } else {
                tokens.push(Token {
                    token_type: TokenType::StringLit(buf),
                    line: tok_line,
                    col: tok_col,
                    file: map_file.clone(),
                });
            }
            continue;
        }

        if chars[index] == '\'' {
            let char_start = token_start;
            map_col += 1;
            index += 1;
            if index >= chars.len() {
                emit_error(
                    program,
                    &map_file,
                    byte_off!(char_start),
                    1,
                    "E008",
                    "unterminated character literal",
                    "char literal started here, never closed",
                    "a char literal must contain exactly one character: `'a'`, `'\\n'`",
                );
                had_error = true;
                continue;
            }
            if chars[index] == '\'' {
                emit_error(
                    program,
                    &map_file,
                    byte_off!(char_start),
                    2,
                    "E009",
                    "empty character literal `''`",
                    "no character inside the literal",
                    "a char literal must contain exactly one character, e.g. `'a'`",
                );
                had_error = true;
                map_col += 1;
                index += 1;
                continue;
            }
            let char_val = if chars[index] == '\\' {
                if index + 1 >= chars.len() {
                    emit_error(
                        program,
                        &map_file,
                        byte_off!(index),
                        1,
                        "E006",
                        "unterminated escape sequence in char literal",
                        "escape started here",
                        "valid escapes: `\\n`, `\\t`, `\\r`, `\\\\`, `\\'`, `\\0`",
                    );
                    had_error = true;
                    '\0'
                } else {
                    map_col += 1;
                    index += 1;
                    let c = chars[index];
                    map_col += 1;
                    index += 1;
                    match c {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '\'' => '\'',
                        '0' => '\0',
                        _ => {
                            emit_error(
                                program,
                                &map_file,
                                byte_off!(index - 2),
                                2,
                                "E006",
                                &format!("unknown escape sequence `\\{c}` in char literal"),
                                "invalid escape",
                                "valid escapes: `\\n`, `\\t`, `\\r`, `\\\\`, `\\'`, `\\0`",
                            );
                            had_error = true;
                            c
                        }
                    }
                }
            } else {
                let c = chars[index];
                map_col += 1;
                index += 1;
                c
            };
            if index < chars.len() && chars[index] == '\'' {
                map_col += 1;
                index += 1;
                tokens.push(Token {
                    token_type: TokenType::CharLit(char_val),
                    line: tok_line,
                    col: tok_col,
                    file: map_file.clone(),
                });
            } else if index < chars.len() && chars[index] != '\'' {
                let extra_start = index;
                while index < chars.len() && chars[index] != '\'' && chars[index] != '\n' {
                    map_col += 1;
                    index += 1;
                }
                let extra_len = index - extra_start + 1;
                emit_error(program, &map_file, byte_off!(char_start), extra_len, "E010",
                    "character literal contains more than one character", "too many characters",
                    "a char literal holds exactly one character; for strings use double quotes: `\"...\"`");
                had_error = true;
                if index < chars.len() && chars[index] == '\'' {
                    map_col += 1;
                    index += 1;
                }
            } else {
                emit_error(
                    program,
                    &map_file,
                    byte_off!(char_start),
                    1,
                    "E008",
                    "unterminated character literal",
                    "char literal started here, never closed",
                    "close the literal with a single quote: `'a'`",
                );
                had_error = true;
            }
            continue;
        }

        if is_operator_char(chars[index]) {
            if index + 1 < chars.len() {
                let two = format!("{}{}", chars[index], chars[index + 1]);
                let result = operator_map(&two);
                if !matches!(result, TokenType::NoMatch) {
                    tokens.push(Token {
                        token_type: result,
                        line: tok_line,
                        col: tok_col,
                        file: map_file.clone(),
                    });
                    map_col += 2;
                    index += 2;
                    continue;
                }
            }
            let one = chars[index].to_string();
            let result = operator_map(&one);
            if !matches!(result, TokenType::NoMatch) {
                tokens.push(Token {
                    token_type: result,
                    line: tok_line,
                    col: tok_col,
                    file: map_file.clone(),
                });
            } else {
                emit_error(program, &map_file, byte_off!(token_start), 1, "E011",
                    &format!("unexpected operator character `{}`", chars[index]),
                    "not a valid operator",
                    "check the operator list; assignment uses `=`, equality uses `==`, not-equal uses `~=`");
                had_error = true;
            }
            map_col += 1;
            index += 1;
            continue;
        }

        let mut buffer = String::new();
        let buf_start = index;
        let buf_line = map_line;
        let buf_col = map_col;
        while index < chars.len() {
            let c = chars[index];
            if c.is_whitespace() || c == '!' || c == ':' || c == '$' {
                break;
            }
            if c == '.' {
                let buf_numeric = buffer
                    .chars()
                    .next()
                    .map(|ch| ch.is_ascii_digit())
                    .unwrap_or(false);
                let next_digit = chars
                    .get(index + 1)
                    .map(|&nc| nc.is_ascii_digit())
                    .unwrap_or(false);
                if buf_numeric && next_digit {
                    buffer.push(c);
                    map_col += 1;
                    index += 1;
                    continue;
                } else {
                    break;
                }
            }
            if (c == 'e' || c == 'E') && buffer.contains('.') {
                let buf_numeric = buffer
                    .chars()
                    .next()
                    .map(|ch| ch.is_ascii_digit())
                    .unwrap_or(false);
                if buf_numeric {
                    match chars.get(index + 1).copied() {
                        Some(d) if d.is_ascii_digit() => {
                            buffer.push(c);
                            map_col += 1;
                            index += 1;
                            continue;
                        }
                        Some(s @ '-') | Some(s @ '+')
                            if chars.get(index + 2).map_or(false, |d| d.is_ascii_digit()) =>
                        {
                            buffer.push(c);
                            buffer.push(s);
                            map_col += 2;
                            index += 2;
                            continue;
                        }
                        _ => {}
                    }
                }
            }
            if is_operator_char(c) {
                break;
            }
            buffer.push(c);
            map_col += 1;
            index += 1;
        }

        if buffer.is_empty() {
            let c = chars[index];
            emit_error(
                program,
                &map_file,
                byte_off!(token_start),
                1,
                "E012",
                &format!("unexpected character `{}`", c),
                "not a valid token start",
                &format!(
                    "the character `{c}` (U+{:04X}) is not valid here; \
                          remove it or check for a copy-paste artefact",
                    c as u32
                ),
            );
            had_error = true;
            map_col += 1;
            index += 1;
            continue;
        }

        let tt = if buffer.starts_with("0b") && buffer.len() > 2 {
            if buffer[2..].chars().any(|c| c != '0' && c != '1') {
                emit_error(
                    program,
                    &map_file,
                    byte_off!(buf_start),
                    buffer.len(),
                    "E013",
                    &format!("invalid binary literal `{}`", buffer),
                    "contains non-binary digit",
                    "binary literals may only contain `0` and `1`, e.g. `0b1010`",
                );
                had_error = true;
                TokenType::NoMatch
            } else {
                classify_buffer(&buffer)
            }
        } else if buffer.starts_with("0o") && buffer.len() > 2 {
            if buffer[2..].chars().any(|c| !('0'..='7').contains(&c)) {
                emit_error(
                    program,
                    &map_file,
                    byte_off!(buf_start),
                    buffer.len(),
                    "E014",
                    &format!("invalid octal literal `{}`", buffer),
                    "contains non-octal digit",
                    "octal literals may only contain digits 0–7, e.g. `0o755`",
                );
                had_error = true;
                TokenType::NoMatch
            } else {
                classify_buffer(&buffer)
            }
        } else if buffer.starts_with("0x") && buffer.len() > 2 {
            if buffer[2..].chars().any(|c| !c.is_ascii_hexdigit()) {
                emit_error(
                    program,
                    &map_file,
                    byte_off!(buf_start),
                    buffer.len(),
                    "E015",
                    &format!("invalid hexadecimal literal `{}`", buffer),
                    "contains non-hex character",
                    "hex literals use digits 0–9 and letters A–F, e.g. `0xFF`",
                );
                had_error = true;
                TokenType::NoMatch
            } else {
                classify_buffer(&buffer)
            }
        } else {
            let tt = classify_buffer(&buffer);
            if matches!(tt, TokenType::NoMatch) {
                let hint = if buffer.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    format!(
                        "`{buffer}` looks like a number but contains non-numeric characters; \
                             identifiers cannot start with a digit"
                    )
                } else {
                    format!("`{buffer}` is not a valid identifier or literal; \
                             identifiers must start with a letter or `_` and contain only letters, digits, and `_`")
                };
                emit_error(
                    program,
                    &map_file,
                    byte_off!(buf_start),
                    buffer.len(),
                    "E016",
                    &format!("unrecognised token `{}`", buffer),
                    "cannot be tokenised",
                    &hint,
                );
                had_error = true;
            }
            tt
        };

        if !matches!(tt, TokenType::NoMatch) {
            tokens.push(Token {
                token_type: tt,
                line: buf_line,
                col: buf_col,
                file: map_file.clone(),
            });
        }
    }

    if had_error {
        let display_file = std::path::Path::new(source_file)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(source_file);
        eprintln!(
            "\x1b[1;31maborting\x1b[0m: lexical error(s) in `{display_file}`; \
                   fix the above before continuing\n"
        );
        std::process::exit(1);
    }

    tokens
}
