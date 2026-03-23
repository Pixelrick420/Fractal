#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]
use std::io::{self, BufRead, Write};

fn __fractal_fmt_float(v: f64) -> String {
    if v.fract() == 0.0 && v.is_finite() {
        format!("{:.1}", v)
    } else {
        format!("{}", v)
    }
}

pub mod fractal_bubblesort {
    use super::*;

    pub fn fractal_bubble(mut fractal_a: &mut Vec<i64>) {
        let mut fractal_n: i64 = 0_i64;
        fractal_n = 5_i64;
        {
            let mut fractal_i: i64 = 0_i64;
            while fractal_i > (-10_i64) {
                { print!("2"); io::stdout().flush().unwrap(); };
                fractal_i += (-1_i64);
            }
        }
        let mut fractal_temp: i64 = 0_i64;
        {
            let mut fractal_i: i64 = 0_i64;
            while fractal_i < (fractal_n - 1_i64) {
                {
                    let mut fractal_j: i64 = 0_i64;
                    while fractal_j < ((fractal_n - 1_i64) - fractal_i) {
                        if (fractal_a[fractal_j as usize] > fractal_a[(fractal_j + 1_i64) as usize]) {
                            fractal_temp = fractal_a[fractal_j as usize];
                            let __idx_0 = fractal_j as usize;
                            fractal_a[__idx_0] = fractal_a[(fractal_j + 1_i64) as usize];
                            let __idx_1 = (fractal_j + 1_i64) as usize;
                            fractal_a[__idx_1] = fractal_temp;
                        }
                        fractal_j += 1_i64;
                    }
                }
                fractal_i += 1_i64;
            }
        }
    }

}

fn main() {
    let mut fractal_b: Vec<i64> = vec![5_i64, 4_i64, 1_i64, 2_i64, 1_i64];
    fractal_bubblesort::fractal_bubble(unsafe { &mut *(&mut fractal_b as *mut _) });
}
