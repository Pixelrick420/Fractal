use std::env;
use std::fs;
use std::io::ErrorKind;
use std::process;

use fractal::compiler::lexer::Token;
use fractal::compiler::parser::pretty_print_root;
use fractal::compiler::semanter;
use fractal::compiler::{lexer, parser, preprocessor};

fn print_error(msg: &str) {
    eprintln!("\x1b[1;31mError:\x1b[0m {}", msg);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
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
    println!("Processed:\n{}", processed_program);
    let tokens: Vec<Token> = lexer::tokenize(&processed_program);

    for token in &tokens {
        println!("{:?}", token);
    }

    match parser::parse(tokens) {
        Ok(node) => {
            pretty_print_root(&node);
            semanter::analyse(&node);
        }
        Err(err) => eprintln!("Parse error: {:?}", err),
    }
}
