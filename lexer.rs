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
    Start,              // start of program
    End,                // end of program
    Exit,               // exit token
    UIntLit(u64),       // unsigned integer = u64
    SIntLit(i64),       // signed integer = i64
    EndL,               // semi colon
    NoMatch,            // dummy for no match
    Identifier(String), // identifier
    If,                 // if
    Else,               // else (also could be used as else if)
    For,                // for loops
    While,              // while loops
    Plus,
    Minus,
    Star,
    Equals,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Greater,
    Less,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
}

fn is_operator(c: char) -> bool {
    return matches!(
        c,
        '+' | '-'
            | '*'
            | '/'
            | '%'
            | '&'
            | '|'
            | '{'
            | '}'
            | '~'
            | '^'
            | '='
            | '('
            | ')'
            | '>'
            | '<'
            | '$'
            | ';'
    );
}

fn is_literal_of_type<T>(s: &str) -> bool
where
    T: FromStr,
{
    !s.is_empty() && s.parse::<T>().is_ok()
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

fn operator_map(s: &str) -> TokenType {
    match s {
        ";" => TokenType::EndL,
        "+" => TokenType::Plus,
        "-" => TokenType::Minus,
        "*" => TokenType::Star,
        "=" => TokenType::Equals,
        "(" => TokenType::LParen,
        ")" => TokenType::RParen,
        "{" => TokenType::LBrace,
        "}" => TokenType::RBrace,
        ">" => TokenType::Greater,
        "<" => TokenType::Less,
        _ => TokenType::NoMatch,
    }
}

fn keyword_map(s: &str) -> TokenType {
    match s {
        "!start" => TokenType::Start,
        "!end" => TokenType::End,
        "!exit" => TokenType::Exit,
        "!if" => TokenType::If,
        "!else" => TokenType::Else,
        "!for" => TokenType::For,
        "!while" => TokenType::While,
        _ => TokenType::NoMatch,
    }
}

fn handle_token(buffer: &String) -> Token {
    if (buffer.len() == 1 && is_operator(buffer.chars().next().unwrap())) {
        return (Token {
            token_type: operator_map(buffer),
        });
    } else if (buffer.starts_with('!')) {
        return (Token {
            token_type: keyword_map(&buffer),
        });
    } else if (is_literal_of_type::<i64>(&buffer)) {
        return (Token {
            token_type: TokenType::SIntLit(buffer.parse::<i64>().expect("")),
        });
    } else if (is_identifier(&buffer)) {
        return (Token {
            token_type: TokenType::Identifier(buffer.clone()),
        });
    }
    return (Token {
        token_type: TokenType::NoMatch,
    });
}

pub fn tokenize(program: &str) -> Vec<Token> {
    let chars: Vec<char> = program.chars().collect();
    let mut tokens: Vec<Token> = Vec::new();
    let mut index: usize = 0;
    let mut buffer: String = String::new();

    while (index < chars.len()) {
        while (index < chars.len()) && (chars[index].is_whitespace()) {
            index += 1;
            continue;
        }

        while (index < chars.len()) && (!chars[index].is_whitespace()) {
            if (is_operator(chars[index])) {
                if (buffer.len() > 0) {
                    tokens.push(handle_token(&buffer));
                    buffer.clear();
                }

                buffer.push(chars[index]);
                tokens.push(handle_token(&buffer));
                buffer.clear();
                index += 1;
                break;
            }

            buffer.push(chars[index]);
            index += 1;
        }

        if (buffer.len() > 0) {
            tokens.push(handle_token(&buffer));
            buffer.clear();
        }
    }
    return tokens;
}
