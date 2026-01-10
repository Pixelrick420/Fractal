#![allow(unused_parens)]
#![allow(unused)]
#![allow(dead_code)]

use lexer::Token;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::process;
use std::process::Command;

mod lexer;

// fn assemble_and_link() -> Result<(), String> {
//     let nasm_output = Command::new("nasm")
//         .args(&["-f", "elf64", "output.asm", "-o", "output.o"])
//         .output()
//         .map_err(|e| format!("Failed to run nasm: {}", e))?;

//     if !nasm_output.status.success() {
//         return Err(format!(
//             "NASM assembly failed:\n{}",
//             String::from_utf8_lossy(&nasm_output.stderr)
//         ));
//     }

//     let ld_output = Command::new("ld")
//         .args(&["output.o", "-o", "output"])
//         .output()
//         .map_err(|e| format!("Failed to run ld: {}", e))?;

//     if !ld_output.status.success() {
//         return Err(format!(
//             "Linking failed:\n{}",
//             String::from_utf8_lossy(&ld_output.stderr)
//         ));
//     }
//     Ok(())
// }

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

    let tokens: Vec<Token> = lexer::tokenize(&contents);
    println!("{:?}", tokens)
    // let tree: Result<Program, String> = parser::parse(tokens);
    // let res: Result<Program, String> = semanter::analyze(tree);

    // match res {
    //     Ok(program) => {
    //         let code: String = codegen::generate(&program);
    //         fs::write("output.asm", code).expect("Failed to write assembly");
    //     }

    //     Err(e) => {
    //         eprintln!("Error: {}", e);
    //         process::exit(1);
    //     }
    // }

    // match assemble_and_link() {
    //     Ok(()) => {
    //         // pass
    //     }
    //     Err(e) => {
    //         eprintln!("Compilation failed: {}", e);
    //         process::exit(1);
    //     }
    // }
}
