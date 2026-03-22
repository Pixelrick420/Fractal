#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]
use std::io::{self, BufRead, Write};

pub fn fibonacci(mut n: i64) -> i64 {
    if (n <= 1_i64) {
        return n;
    } else {
        return (fibonacci((n - 1_i64)) + fibonacci((n - 2_i64)));
    }
}

pub fn main2() {
    { print!("Fibonacci series up to 10:"); io::stdout().flush().unwrap(); };
    {
        let mut i: i64 = 0_i64;
        while i < 10_i64 {
            { print!("{}", fibonacci(i)); io::stdout().flush().unwrap(); };
            i += 1_i64;
        }
    }
}

fn main() {
    main2();
}
