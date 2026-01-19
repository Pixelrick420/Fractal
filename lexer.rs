#![allow(unused_parens)]
#![allow(unused)]
#![allow(dead_code)]

use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::process;
use std::str::FromStr;

#[derive(Debug)]
pub enum TokenType {
    // keywords
    Start,
    End,
    Exit,
    If,
    Else,
    For,
    While,
    Func,
    Return,
    Struct,

    // datatypes
    TypeInt,
    TypeFloat,
    TypeChar,
    TypeBoolean,
    TypeArray,
    TypeList,
    TypeStruct,

    // literals
    SIntLit(i64),
    FloatLit(f64),
    CharLit(char),
    StringLit(String),
    BoolLit(bool),
    Null,

    // identifiers
    Identifier(String),

    // arithmetic operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // logical operators
    Ampersand,
    Pipe,
    Caret,
    Tilde,

    // assignment operators
    Equals,
    PlusEquals,
    MinusEquals,
    StarEquals,
    SlashEquals,
    PercentEquals,
    AmpersandEquals,
    PipeEquals,
    CaretEquals,

    // comparison operators
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
    EqualsEquals,
    TildeEquals,

    // special operators
    Arrow,
    Dot,
    Comma,

    // delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LAngle,
    RAngle,

    // other
    EndL,
    NoMatch,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
}

fn is_operator_char(c: char) -> bool {
    return matches!(
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
    );
}

fn is_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let chars = s.as_bytes();
    if !((chars[0] as char).is_alphabetic() || (chars[0] as char) == '_') {
        return false;
    }

    for c in chars {
        let cur: char = (*c as char);
        if !(cur.is_alphanumeric() || cur == '_') {
            return false;
        }
    }
    return true;
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
    } else {
        if let Ok(val) = s.parse::<i64>() {
            return TokenType::SIntLit(val);
        }
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
        _ => TokenType::NoMatch,
    }
}

fn keyword_map(s: &str) -> TokenType {
    match s {
        "start" => TokenType::Start,
        "end" => TokenType::End,
        "exit" => TokenType::Exit,
        "if" => TokenType::If,
        "else" => TokenType::Else,
        "for" => TokenType::For,
        "while" => TokenType::While,
        "func" => TokenType::Func,
        "return" => TokenType::Return,
        "struct" => TokenType::Struct,
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
        _ => TokenType::NoMatch,
    }
}

fn handle_token(buffer: &String) -> Token {
    match buffer.as_str() {
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
        "NULL" => {
            return Token {
                token_type: TokenType::Null,
            }
        }
        _ => {}
    }

    let operator_result = operator_map(&buffer);
    if !matches!(operator_result, TokenType::NoMatch) {
        return Token {
            token_type: operator_result,
        };
    }

    let number_result = parse_number_literal(&buffer);
    if !matches!(number_result, TokenType::NoMatch) {
        return Token {
            token_type: number_result,
        };
    }

    if is_identifier(&buffer) {
        return Token {
            token_type: TokenType::Identifier(buffer.clone()),
        };
    }

    Token {
        token_type: TokenType::NoMatch,
    }
}

pub fn tokenize(program: &str) -> Vec<Token> {
    let chars: Vec<char> = program.chars().collect();
    let mut tokens: Vec<Token> = Vec::new();
    let mut index: usize = 0;

    while index < chars.len() {
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }

        if index >= chars.len() {
            break;
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
            if !matches!(result, TokenType::NoMatch) {
                tokens.push(Token { token_type: result });
            } else {
                tokens.push(Token {
                    token_type: TokenType::NoMatch,
                });
            }
            continue;
        }

        if chars[index] == ':' {
            index += 1;
            let mut buffer = String::new();
            while index < chars.len()
                && !chars[index].is_whitespace()
                && !is_operator_char(chars[index])
            {
                buffer.push(chars[index]);
                index += 1;
            }

            let result = type_map(&buffer);
            if !matches!(result, TokenType::NoMatch) {
                tokens.push(Token { token_type: result });
            } else {
                tokens.push(Token {
                    token_type: TokenType::NoMatch,
                });
            }
            continue;
        }

        if chars[index] == '"' {
            index += 1;
            let mut string_buf = String::new();
            while index < chars.len() && chars[index] != '"' {
                if chars[index] == '\\' && index + 1 < chars.len() {
                    index += 1;
                    match chars[index] {
                        'n' => string_buf.push('\n'),
                        't' => string_buf.push('\t'),
                        'r' => string_buf.push('\r'),
                        '\\' => string_buf.push('\\'),
                        '"' => string_buf.push('"'),
                        _ => string_buf.push(chars[index]),
                    }
                } else {
                    string_buf.push(chars[index]);
                }
                index += 1;
            }
            if index < chars.len() {
                index += 1;
            }
            tokens.push(Token {
                token_type: TokenType::StringLit(string_buf),
            });
            continue;
        }

        if chars[index] == '\'' {
            index += 1;
            let mut char_val = '\0';
            if index < chars.len() {
                if chars[index] == '\\' && index + 1 < chars.len() {
                    index += 1;
                    char_val = match chars[index] {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '\'' => '\'',
                        _ => chars[index],
                    };
                } else {
                    char_val = chars[index];
                }
                index += 1;
            }
            if index < chars.len() && chars[index] == '\'' {
                index += 1;
            }
            tokens.push(Token {
                token_type: TokenType::CharLit(char_val),
            });
            continue;
        }

        if is_operator_char(chars[index]) {
            let mut op_buf = String::new();
            op_buf.push(chars[index]);

            if index + 1 < chars.len() {
                let two_char = format!("{}{}", chars[index], chars[index + 1]);
                let result = operator_map(&two_char);
                if !matches!(result, TokenType::NoMatch) {
                    tokens.push(Token { token_type: result });
                    index += 2;
                    continue;
                }
            }

            let result = operator_map(&op_buf);
            if !matches!(result, TokenType::NoMatch) {
                tokens.push(Token { token_type: result });
                index += 1;
                continue;
            }

            index += 1;
            continue;
        }

        let mut buffer = String::new();
        while index < chars.len()
            && !chars[index].is_whitespace()
            && !is_operator_char(chars[index])
            && chars[index] != '!'
            && chars[index] != ':'
        {
            buffer.push(chars[index]);
            index += 1;
        }

        if !buffer.is_empty() {
            tokens.push(handle_token(&buffer));
        }
    }

    return tokens;
}
