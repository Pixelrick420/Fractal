#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]
use std::io::{self, BufRead, Write};

fn __fractal_fmt_float(v: f64) -> String {
    if v.fract() == 0.0 && v.is_finite() {
        format!("{:.1}", v)
    } else {
        format!("{}", v)
    }
}

pub fn fractal_add(mut fractal_a: i64, mut fractal_b: i64) -> i64 {
    return (fractal_a + fractal_b);
}

#[derive(Debug, Clone, Default)]
pub struct FractalPoint {
    pub x: Option<i64>,
    pub y: Option<i64>,
}

pub mod fractal_constants {
    use super::*;

    pub static fractal_golden_ratio: f64 = 1.618_f64;
    pub static fractal_euler_constant: f64 = 0.577_f64;
    pub static fractal_max_iterations: i64 = 10000_i64;
}

pub mod fractal_math {
    use super::*;

    pub fn fractal_calculate() -> f64 {
        return fractal_constants::fractal_golden_ratio;
    }

    pub fn fractal_sqrt(mut fractal_a: f64) -> f64 {
        let mut fractal_guess: f64 = (fractal_a / 2.0_f64);
        let mut fractal_c: f64 = 2.0_f64;
        let mut fractal_error: f64 = 0.1_f64;
        let mut fractal_temp: f64 = (fractal_guess - (fractal_a / fractal_guess));
        if (fractal_temp < 0.0_f64) {
            fractal_temp = (-fractal_temp);
        }
        while (fractal_temp > fractal_error) {
            fractal_guess = ((fractal_guess + (fractal_a / fractal_guess)) / fractal_c);
            fractal_temp = (fractal_guess - (fractal_a / fractal_guess));
            if (fractal_temp < 0.0_f64) {
                fractal_temp = (-fractal_temp);
            }
        }
        return fractal_guess;
    }

    pub static fractal_pi: f64 = 3.14159_f64;
    pub static fractal_e: f64 = 2.71828_f64;
    pub static fractal_phi: f64 = 1.61_f64;
}

fn main() {
    let mut fractal_a1: i64 = (10_i64 + 5_i64);
    let mut fractal_a2: i64 = (10_i64 - 5_i64);
    let mut fractal_a3: i64 = (10_i64 * 5_i64);
    let mut fractal_a4: i64 = (10_i64 / 5_i64);
    let mut fractal_a5: i64 = (10_i64 % 3_i64);
    { print!("arithmetic: {} {} {} {} {}", fractal_a1, fractal_a2, fractal_a3, fractal_a4, fractal_a5); io::stdout().flush().unwrap(); };
    let mut fractal_f1: f64 = (10.0_f64 + 5.0_f64);
    let mut fractal_f2: f64 = (10.0_f64 - 5.0_f64);
    let mut fractal_f3: f64 = (10.0_f64 * 5.0_f64);
    let mut fractal_f4: f64 = (10.0_f64 / 2.0_f64);
    { print!("float: {} {} {} {}", __fractal_fmt_float(fractal_f1), __fractal_fmt_float(fractal_f2), __fractal_fmt_float(fractal_f3), __fractal_fmt_float(fractal_f4)); io::stdout().flush().unwrap(); };
    let mut fractal_t: bool = true;
    let mut fractal_f: bool = false;
    let mut fractal_band: bool = (fractal_t && fractal_f);
    let mut fractal_bor: bool = (fractal_t || fractal_f);
    let mut fractal_bnot: bool = (!fractal_t);
    { print!("boolean: {} {} {}", fractal_band, fractal_bor, fractal_bnot); io::stdout().flush().unwrap(); };
    let mut fractal_gt: bool = (5_i64 > 3_i64);
    let mut fractal_lt: bool = (5_i64 < 3_i64);
    let mut fractal_eq: bool = (5_i64 == 5_i64);
    let mut fractal_ne: bool = (5_i64 != 3_i64);
    { print!("compare: {} {} {} {}", fractal_gt, fractal_lt, fractal_eq, fractal_ne); io::stdout().flush().unwrap(); };
    let mut fractal_x: i64 = (5_i64 & 3_i64);
    let mut fractal_y: i64 = (5_i64 | 3_i64);
    let mut fractal_z: i64 = (5_i64 ^ 3_i64);
    { print!("bitwise: {} {} {}", fractal_x, fractal_y, fractal_z); io::stdout().flush().unwrap(); };
    let mut fractal_shl: i64 = (5_i64 << 2_i64);
    let mut fractal_shr: i64 = (10_i64 >> 1_i64);
    { print!("shift: {} {}", fractal_shl, fractal_shr); io::stdout().flush().unwrap(); };
    let mut fractal_arr: [i64; 5] = [1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let mut fractal_first: i64 = fractal_arr[0_i64 as usize];
    let mut fractal_last: i64 = fractal_arr[4_i64 as usize];
    { print!("array: {} {}", fractal_first, fractal_last); io::stdout().flush().unwrap(); };
    let mut fractal_lst: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    fractal_lst.push(4_i64.clone());
    let mut fractal_popped: i64 = fractal_lst.pop().unwrap();
    let mut fractal_lfirst: i64 = fractal_lst[0_i64 as usize];
    { print!("list: {} {}", fractal_lfirst, fractal_popped); io::stdout().flush().unwrap(); };
    let mut fractal_cnt: i64 = 0_i64;
    if (fractal_cnt == 0_i64) {
        { print!("if works"); io::stdout().flush().unwrap(); };
    } else if (fractal_cnt > 0_i64) {
        { print!("elif"); io::stdout().flush().unwrap(); };
    } else {
        { print!("else"); io::stdout().flush().unwrap(); };
    }
    {
        let mut fractal_i: i64 = 0_i64;
        while fractal_i < 3_i64 {
            { print!("for {}", fractal_i); io::stdout().flush().unwrap(); };
            fractal_i += 1_i64;
        }
    }
    let mut fractal_w: i64 = 0_i64;
    while (fractal_w < 2_i64) {
        fractal_w += 1_i64;
    }
    let mut fractal_sum: i64 = 0_i64;
    {
        let mut fractal_j: i64 = 0_i64;
        while fractal_j < 10_i64 {
            if (fractal_j == 5_i64) {
                break;
            }
            fractal_sum += fractal_j;
            fractal_j += 1_i64;
        }
    }
    { print!("break sum={}", fractal_sum); io::stdout().flush().unwrap(); };
    let mut fractal_result: i64 = fractal_add(3_i64, 4_i64);
    { print!("func result={}", fractal_result); io::stdout().flush().unwrap(); };
    let mut fractal_p: Option<Box<FractalPoint>> = Some(Box::new(FractalPoint { x: Some(1_i64), y: Some(2_i64) }));
    { print!("struct: {} {}", fractal_p.as_ref().unwrap().x.unwrap(), fractal_p.as_ref().unwrap().y.unwrap()); io::stdout().flush().unwrap(); };
    let mut fractal_ic: i64 = (3.7_f64 as i64);
    let mut fractal_fc: f64 = (5_i64 as f64);
    let mut fractal_bc: bool = (1_i64 != 0_i64);
    let mut fractal_cc: char = (char::from_u32(65_i64 as u32).unwrap());
    { print!("casts: {} {} {} {}", fractal_ic, __fractal_fmt_float(fractal_fc), fractal_bc, fractal_cc); io::stdout().flush().unwrap(); };
    let mut fractal_absv: i64 = (-5_i64).abs();
    let mut fractal_sq: f64 = (16.0_f64 as f64).sqrt();
    let mut fractal_mx: i64 = 3_i64.max(7_i64);
    let mut fractal_mn: i64 = 3_i64.min(7_i64);
    let mut fractal_fl: i64 = (3.7_f64 as f64).floor() as i64;
    let mut fractal_cl: i64 = (3.2_f64 as f64).ceil() as i64;
    { print!("builtins: {} {} {} {} {} {}", fractal_absv, __fractal_fmt_float(fractal_sq), fractal_mx, fractal_mn, fractal_fl, fractal_cl); io::stdout().flush().unwrap(); };
    let mut fractal_sr: f64 = (16.0_f64 as f64).sqrt();
    { print!("import sqrt: {}", __fractal_fmt_float(fractal_sr)); io::stdout().flush().unwrap(); };
    { print!("all tests passed"); io::stdout().flush().unwrap(); };
}
