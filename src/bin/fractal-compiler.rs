use std::env;
use std::fs;
use std::io::ErrorKind;
use std::process;

use fractal::compiler::semanter::analyze;
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

    let source_file = &args[1];

    let contents = match fs::read_to_string(source_file) {
        Ok(data) => data,
        Err(err) => {
            match err.kind() {
                ErrorKind::NotFound => {
                    print_error(&format!("file not found: `{source_file}`"));
                }
                ErrorKind::PermissionDenied => {
                    print_error(&format!("permission denied reading `{source_file}`"));
                }
                _ => {
                    print_error(&format!("could not read `{source_file}`: {err}"));
                }
            }
            process::exit(1);
        }
    };

    let processed_program = preprocessor::preprocess(&contents, source_file);
    let tokens = lexer::tokenize_with_source(&processed_program, source_file);

    match parser::parse_with_source(tokens, source_file) {
        Ok(node) => {
            let result = analyze(&node);
            result.print_symbol_table();
            result.print_errors();
            if result.has_errors() {
                process::exit(1);
            }
        }
        Err(err) => {
            err.emit(&processed_program);
            process::exit(1);
        }
    }
}
