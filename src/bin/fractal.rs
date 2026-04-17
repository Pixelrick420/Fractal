use std::env;
use std::process::{self, Command};

fn main() {
    // Pass all arguments straight through to fractal-compiler.
    // This binary's only job is to be named "fractal" so users
    // type "fractal foo.fr" instead of "fractal-compiler foo.fr".
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: fractal [--debug] <path/to/file.fr>");
        process::exit(1);
    }

    // Find fractal-compiler sitting next to this binary
    let mut compiler = env::current_exe()
        .expect("could not find current executable path");
    compiler.pop(); // remove "fractal" filename, keep the directory

    #[cfg(target_os = "windows")]
    compiler.push("fractal-compiler.exe");
    #[cfg(not(target_os = "windows"))]
    compiler.push("fractal-compiler");

    if !compiler.exists() {
        eprintln!("Error: could not find `fractal-compiler` next to this binary.");
        eprintln!("Make sure you ran `cargo build --release`.");
        process::exit(1);
    }

    let status = Command::new(&compiler)
        .args(&args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to launch fractal-compiler: {}", e);
            process::exit(1);
        });

    process::exit(status.code().unwrap_or(1));
}