#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]
use std::io::{self, BufRead, Write};

fn __fractal_fmt_float(v: f64) -> String {
    if v.fract() == 0.0 && v.is_finite() {
        format!("{:.1}", v)
    } else {
        format!("{}", v)
    }
}

pub fn fractal_power(mut fractal_m: i64, mut fractal_pow: i64) -> i64 {
    let mut fractal_ans: i64 = 1_i64;
    {
        let mut fractal_i: i64 = 0_i64;
        while fractal_i < fractal_pow {
            fractal_ans *= fractal_m;
            fractal_i += 1_i64;
        }
    }
    return fractal_ans;
}

fn main() {
    let mut fractal_n: i64 = 0_i64;
    { print!("Enter number:"); io::stdout().flush().unwrap(); };
    { let mut __ln = String::new(); io::stdin().lock().read_line(&mut __ln).unwrap(); let mut __toks = __ln.trim().split_whitespace(); fractal_n = __toks.next().unwrap_or("").parse::<i64>().unwrap_or(0_i64); };
    let mut fractal_n2: i64 = fractal_n;
    let mut fractal_sum: i64 = 0_i64;
    let mut fractal_temp: i64 = 0_i64;
    let mut fractal_count: i64 = 0_i64;
    while (fractal_n2 > 0_i64) {
        fractal_count += 1_i64;
        fractal_n2 /= 10_i64;
    }
    fractal_n2 = fractal_n;
    while (fractal_n2 > 0_i64) {
        fractal_temp = (fractal_n2 % 10_i64);
        fractal_sum += fractal_power(fractal_temp, fractal_count);
        fractal_n2 /= 10_i64;
    }
    if (fractal_sum == fractal_n) {
        { print!("{} is armstrong\n", fractal_n); io::stdout().flush().unwrap(); };
    } else {
        { print!("{} is not armstrong\n", fractal_n); io::stdout().flush().unwrap(); };
    }
}
