#![allow(unused_parens)]
#![allow(unused)]
#![allow(dead_code)]

use std::env;
use std::fs;
use std::io::ErrorKind;
use std::process;
use std::process::Command;

use fractal::compiler::lexer::Token;
<<<<<<< HEAD
use fractal::compiler::{lexer, preprocessor, generated_parser};
=======
use fractal::compiler::{lexer, parser, preprocessor};
>>>>>>> ae479cafb2678254eeb706343b06168c6a683a0b

fn print_error(msg: &str) {
    eprintln!("\x1b[1;31mError:\x1b[0m {}", msg);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if (args.len() != 2) {
        print_error(&format!("Usage: {} <path/to/file.fr>", &args[0]));
        process::exit(1);
    }

    let contents = match fs::read_to_string(&args[1]) {
        Ok(data) => data,
        Err(err) => {
            match err.kind() {
                ErrorKind::NotFound => {
                    print_error("File not found");
                }
                ErrorKind::PermissionDenied => {
                    print_error("Permission denied");
                }
                _ => {
                    print_error(&format!("Reading file: {}", err));
                }
            }
            process::exit(1);
        }
    };

    let processed_program: String = preprocessor::preprocess(&contents, &args[1]);
    //println!("Processed:\n{}", processed_program);
    let tokens: Vec<Token> = lexer::tokenize(&processed_program);

    //println!("\nTokens:");
    //parser::create_tree(tokens);
<<<<<<< HEAD
    // for token in &tokens {
    //      println!("{:?}", token);
    //  }
    let mut parser = generated_parser::Parser::new(tokens);
    let tree = parser.parse_prgm();
    tree.print(0);
=======
    for token in &tokens {
        println!("{:?}", token);
    }
>>>>>>> ae479cafb2678254eeb706343b06168c6a683a0b
}
