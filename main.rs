#![allow(unused_parens)]
#![allow(unused)]
#![allow(dead_code)]

use lexer::Token;

use std::env;
use std::fs;
use std::io::ErrorKind;
use std::process;
use std::process::Command;

mod preprocessor;
mod lexer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if (args.len() != 2) {
        eprintln!("Usage : {} <path/to/file.rs>", &args[0]);
        process::exit(1);
    }
    
    let contents = match fs::read_to_string(&args[1]) {
        Ok(data) => data,
        Err(err) => {
            match err.kind() {
                ErrorKind::NotFound => {
                    eprintln!("Error: file not found");
                }
                ErrorKind::PermissionDenied => {
                    eprintln!("Error: permission denied");
                }
                _ => {
                    eprintln!("Error reading file: {}", err);
                }
            }
            process::exit(1);
        }
    };
    let processed_program: &str = &preprocessor::preprocess(&contents);

    let tokens: Vec<Token> = lexer::tokenize(&processed_program);
    let n: usize = tokens.len();
    for i in 1..n {
        println!("{:?}", tokens[i]);
    }
}
