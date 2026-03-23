#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]
use std::io::{self, BufRead, Write};

fn __fractal_fmt_float(v: f64) -> String {
    if v.fract() == 0.0 && v.is_finite() {
        format!("{:.1}", v)
    } else {
        format!("{}", v)
    }
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
        return fractal_constants::golden_ratio;
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

#[derive(Debug, Clone, Default)]
pub struct FractalNode {
    pub a: Option<i64>,
    pub b: Option<i64>,
    pub arr: Option<[i64; 3]>,
    pub next: Option<Box<FractalNode>>,
}

pub fn fractal_hello(mut fractal_a: i64, mut fractal_b: f64, mut fractal_c: &mut Vec<f64>) -> bool {
    let mut fractal_sqrtOfB: f64 = fractal_math::(fractal_b as f64).sqrt();
    let mut fractal_sqrtOfC: f64 = fractal_math::(fractal_c[0_i64 as usize] as f64).sqrt();
    let __idx_0 = 0_i64 as usize;
    fractal_c[__idx_0] = 0.0_f64;
    return (fractal_sqrtOfB > fractal_sqrtOfC);
}

pub fn fractal_traverse(mut fractal_node: &mut Option<Box<FractalNode>>, mut fractal_values: &mut [Vec<i64>; 100], mut fractal_index: i64) {
    if (fractal_node == None) {
        return;
    }
    let mut fractal_temp: Vec<i64> = vec![fractal_node.as_ref().unwrap().a.unwrap(), fractal_node.as_ref().unwrap().b.unwrap()];
    let __idx_1 = fractal_index as usize;
    fractal_values[__idx_1] = fractal_temp;
    fractal_traverse(&mut fractal_node.as_mut().unwrap().next, fractal_values, (fractal_index + 1_i64));
}

fn main() {
    let mut fractal_a: i64 = 100_i64;
    let mut fractal_b: f64 = 5.0_f64;
    let mut fractal_arr: [i64; 5] = [1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let mut fractal_nums: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    { print!("{}", (fractal_arr[0_i64 as usize] as i64)); io::stdout().flush().unwrap(); };
    if ((fractal_a as f64) > fractal_b) {
        fractal_a = (fractal_a + 1_i64);
    } else {
        fractal_b = (fractal_b + (1_i64 as f64));
    }
    fractal_a = (fractal_math::pi as i64);
    fractal_nums.push(fractal_a.clone());
    {
        let mut fractal_i: i64 = 0_i64;
        while fractal_i < 10_i64 {
            if ((fractal_i % 3_i64) == 0_i64) {
                { print!("{} is a multiple of 3", (fractal_i as i64)); io::stdout().flush().unwrap(); };
                fractal_i += 1_i64;
                continue;
            }
            fractal_i += 1_i64;
        }
    }
    while ((fractal_a as f64) > fractal_b) {
        fractal_a -= 1_i64;
        if (fractal_a == 3_i64) {
            break;
        } else if ((fractal_a % 25_i64) == 0_i64) {
            { print!("Hello: a = {}\n", (fractal_a as i64)); io::stdout().flush().unwrap(); };
        }
    }
    let mut fractal_top: Vec<f64> = vec![5.6_f64, 25.1_f64];
    let mut fractal_res: bool = fractal_hello(fractal_a, fractal_b, unsafe { &mut *(&mut fractal_top as *mut _) });
    { print!("{}\n", __fractal_fmt_float((fractal_top[0_i64 as usize] as f64))); io::stdout().flush().unwrap(); };
    let mut fractal_root: Option<Box<FractalNode>> = Some(Box::new(FractalNode { a: Some(67_i64), b: Some(69_i64), arr: Some([1_i64, 2_i64, 3_i64]), next: None }));
    let mut fractal_cur: Option<Box<FractalNode>> = fractal_root;
    {
        let mut fractal_i: i64 = 0_i64;
        while fractal_i < 5_i64 {
            let mut fractal_node: Option<Box<FractalNode>> = Some(Box::new(FractalNode { a: Some(fractal_i), b: Some((fractal_i * 5_i64)), arr: Some([0_i64, 0_i64, 0_i64]), next: None }));
            let mut fractal_z: i64 = 0_i64;
            {
                let mut fractal_j: i64 = fractal_node.as_ref().unwrap().a.unwrap();
                while fractal_j < fractal_node.as_ref().unwrap().b.unwrap() {
                    let __idx_2 = fractal_z as usize;
                    fractal_node.as_mut().unwrap().arr.as_mut().unwrap()[__idx_2] = fractal_j;
                    fractal_z += 1_i64;
                    fractal_j += 1_i64;
                }
            }
            fractal_cur.as_mut().unwrap().next = Some(Box::new(fractal_node));
            fractal_cur = fractal_node;
            fractal_i += 1_i64;
        }
    }
    fractal_cur.as_mut().unwrap().next = None;
    let mut fractal_values: [Vec<i64>; 100] = [Vec::new(); 100];
    let mut fractal_index: i64 = 0_i64;
    let mut fractal_hehe: i64 = fractal_cur.as_ref().unwrap().a.unwrap();
    fractal_traverse(&mut fractal_root, unsafe { &mut *(&mut fractal_values as *mut _) }, fractal_index);
}
