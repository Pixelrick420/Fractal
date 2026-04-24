use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process;

use fractal::compiler::codegen;
use fractal::compiler::semanter::analyze;
use fractal::compiler::{lexer, parser, preprocessor};

const DEBUG: bool = false;
const DELETE: bool = false;

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

    let (debug_mode, emit_rust_only, source_file_str) = match args.as_slice() {
        [_, f] => (false, false, f.clone()),
        [_, flag, f] if flag == "--debug" => (true, false, f.clone()),
        [_, sub, f] if sub == "debug" => (true, false, f.clone()),
        [_, flag, f] if flag == "--emit-rust" => (false, true, f.clone()),
        _ => {
            print_error(&format!(
                "Usage: {} [--debug | debug | --emit-rust] <path/to/file.fr>",
                &args[0]
            ));
            eprintln!();
            eprintln!("  {}  file.fr              compile normally", &args[0]);
            eprintln!(
                "  {}  debug file.fr         compile with debugger instrumentation",
                &args[0]
            );
            eprintln!(
                "  {}  --debug file.fr        same as above (flag form)",
                &args[0]
            );
            eprintln!(
                "  {}  --emit-rust file.fr    output Rust source to stdout, skip rustc",
                &args[0]
            );
            process::exit(1);
        }
    };

    let source_file = &source_file_str;

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
        section("STAGE 1 - PREPROCESSOR OUTPUT");
        for (i, line) in processed_program.lines().enumerate() {
            eprintln!("{:>4} │ {}", i + 1, line);
        }
    }

    let tokens = lexer::tokenize_with_source(&processed_program, source_file);

    if DEBUG {
        section("STAGE 2 - LEXER TOKENS");
    }

    match parser::parse_with_source(tokens, source_file) {
        Ok(node) => {
            if DEBUG {
                section("STAGE 3 - PARSE TREE");
            }

            let result = analyze(&node);

            if DEBUG {
                section("STAGE 4 - SEMANTIC ANALYSIS");
            }
            result.print_errors();
            if DEBUG {
                result.print_symbol_table();
            }

            if result.has_errors() {
                process::exit(1);
            }

            let debug_jsonl_path = Path::new(source_file)
                .with_extension("debug.jsonl")
                .to_string_lossy()
                .to_string();

            let rs_code = if debug_mode {
                codegen::generate_debug(&node, &result, &debug_jsonl_path)
            } else {
                codegen::generate(&node, &result)
            };

            if DEBUG {
                section("STAGE 5 - GENERATED RUST CODE");
                for (i, line) in rs_code.lines().enumerate() {
                    eprintln!("{:>4} │ {}", i + 1, line);
                }
            }

            // --emit-rust: print Rust source to stdout and exit, skipping rustc
            if emit_rust_only {
                print!("{}", rs_code);
                process::exit(0);
            }

            let out_path = Path::new(source_file).with_extension("rs");

            if let Err(e) = fs::write(&out_path, &rs_code) {
                print_error(&format!("could not write `{}`: {}", out_path.display(), e));
                process::exit(1);
            }

            #[cfg(target_os = "windows")]
            let bin_path = Path::new(source_file).with_extension("exe");
            #[cfg(not(target_os = "windows"))]
            let bin_path = Path::new(source_file).with_extension("");

            if DEBUG {
                section("STAGE 6 - RUSTC");
            }

            let crate_name = Path::new(source_file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("fractal_program")
                .replace('.', "_")
                .replace('-', "_");

            let rustc_status = process::Command::new("rustc")
                .arg(&out_path)
                .arg("--crate-name")
                .arg(&crate_name)
                .arg("-o")
                .arg(&bin_path)
                .arg("-C")
                .arg("opt-level=2")
                .arg("-C")
                .arg("codegen-units=1")
                .arg("-A")
                .arg("warnings")
                .status();

            match rustc_status {
                Ok(status) if status.success() => {
                    if DELETE {
                        let _ = fs::remove_file(&out_path);
                    }
                    let display = bin_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("<binary>");
                    eprintln!("\x1b[1;32m compiled:\x1b[0m `{}`", display);
                    if debug_mode {
                        let meta_path = Path::new(source_file).with_extension("debug-meta");
                        let _ = fs::write(&meta_path, &debug_jsonl_path);
                        eprintln!(
                            "\x1b[1;34m   debug:\x1b[0m snapshots → `{}`",
                            debug_jsonl_path
                        );
                    }
                }
                Ok(_) => {
                    print_error("rustc reported errors - this is likely a compiler bug");
                    eprintln!(
                        " \x1b[1;34m  =\x1b[0m \x1b[1;36mnote\x1b[0m: the generated Rust source has been kept at `{}` for inspection",
                        out_path.display()
                    );
                    eprintln!(
                        " \x1b[1;34m  =\x1b[0m \x1b[1;32mhint\x1b[0m: run `rustc {}` manually to see the full rustc error output",
                        out_path.display()
                    );
                    eprintln!(
                        " \x1b[1;34m  =\x1b[0m \x1b[1;32mhint\x1b[0m: if this is reproducible, please report it as a compiler bug"
                    );
                    eprintln!();
                    process::exit(1);
                }
                Err(e) => {
                    print_error(&format!("could not run `rustc`: {e}"));
                    eprintln!(
                        " \x1b[1;34m  =\x1b[0m \x1b[1;32mhint\x1b[0m: make sure `rustc` is installed and available on your PATH"
                    );
                    eprintln!(
                        " \x1b[1;34m  =\x1b[0m \x1b[1;32mhint\x1b[0m: install Rust from https://rustup.rs if it is not already installed"
                    );
                    eprintln!();
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
