use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process;

use fractal::compiler::codegen;
use fractal::compiler::semanter::analyze;
use fractal::compiler::{lexer, parser, preprocessor};

const DEBUG: bool = false;
const DELETE: bool = true;

fn print_error(msg: &str) {
    eprintln!("\x1b[1;31mError:\x1b[0m {}", msg);
}

fn section(title: &str) {
    let line = "═".repeat(80);
    eprintln!("\n\x1b[1;36m╔{}╗\x1b[0m", line);
    eprintln!("\x1b[1;36m║  {:<78}║\x1b[0m", title);
    eprintln!("\x1b[1;36m╚{}╝\x1b[0m", line);
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

    if DEBUG {
        section("STAGE 1 — PREPROCESSOR OUTPUT");
        for (i, line) in processed_program.lines().enumerate() {
            eprintln!("{:>4} │ {}", i + 1, line);
        }
    }

    let tokens = lexer::tokenize_with_source(&processed_program, source_file);

    if DEBUG {
        section("STAGE 2 — LEXER TOKENS");
    }

    match parser::parse_with_source(tokens, source_file) {
        Ok(node) => {
            if DEBUG {
                section("STAGE 3 — PARSE TREE");
            }

            let result = analyze(&node);

            if DEBUG {
                section("STAGE 4 — SEMANTIC ANALYSIS");
            }
            result.print_errors();
            if DEBUG {
                result.print_symbol_table();
            }

            if result.has_errors() {
                process::exit(1);
            }

            let rs_code = codegen::generate(&node, &result);

            if DEBUG {
                section("STAGE 5 — GENERATED RUST CODE");
                for (i, line) in rs_code.lines().enumerate() {
                    eprintln!("{:>4} │ {}", i + 1, line);
                }
            }

            let out_path = Path::new(source_file).with_extension("rs");

            if let Err(e) = fs::write(&out_path, &rs_code) {
                print_error(&format!("could not write `{}`: {}", out_path.display(), e));
                process::exit(1);
            }

            let bin_path = Path::new(source_file).with_extension("");

            if DEBUG {
                section("STAGE 6 — RUSTC");
            }
            let rustc_status = process::Command::new("rustc")
                .arg(&out_path)
                .arg("-o")
                .arg(&bin_path)
                .arg("-C")
                .arg("opt-level=2")
                .arg("-A")
                .arg("warnings")
                .status();

            if DELETE {
                let _ = fs::remove_file(&out_path);
            }

            match rustc_status {
                Ok(status) if status.success() => {
                    let display = bin_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("<binary>");
                    eprintln!("\x1b[1;32m compiled:\x1b[0m `{}`", display);
                }
                Ok(_) => {
                    print_error("rustc reported errors — compilation failed");
                    if DEBUG && !DELETE {
                        eprintln!(
                            "\x1b[1;33m[debug] the .rs file has been kept at {} for inspection\x1b[0m",
                            out_path.display()
                        );
                    }
                    process::exit(1);
                }
                Err(e) => {
                    print_error(&format!("could not run `rustc`: {e}"));
                    print_error("make sure `rustc` is installed and on your PATH");
                    process::exit(1);
                }
            }
        }
        Err(err) => {
            err.emit(&processed_program);
            process::exit(1);
        }
    }
}
