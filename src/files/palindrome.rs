#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]
use std::io::{self, BufRead, Write};

fn main() {
    println!("{}", "Enter string\n".to_string());
    let mut s: [char; 10] = ['\0'; 10];
    input("{}".to_string(), s);
    {
        let mut i: i64 = 0_i64;
        while i < 10_i64 {
            println!("{} {}", "{}".to_string(), s[i as usize]);
            i += 1_i64;
        }
    }
}
