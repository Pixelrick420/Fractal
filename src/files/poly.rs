#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone, Default)]
pub struct Poly {
    pub co: Option<Vec<f64>>,
    pub degree: Option<i64>,
}

pub fn make_poly(mut degree: i64) -> Option<Box<Poly>> {
    let mut p: Option<Box<Poly>> = None;
    p.as_mut().unwrap().degree = Some(degree);
    p.as_mut().unwrap().co = Some([]);
    {
        let mut i: i64 = 0_i64;
        while i < (degree + 1_i64) {
            p.as_ref().unwrap().co.unwrap().push(0.0_f64);
            i += 1_i64;
        }
    }
    return p;
}

pub fn set_coeff(mut p: Option<Box<Poly>>, mut i: i64, mut v: f64) {
    p.as_ref().unwrap().co[i as usize] = v;
}

pub fn eval_poly(mut p: Option<Box<Poly>>, mut x: f64) -> f64 {
    let mut result: f64 = 0.0_f64;
    let mut xpow: f64 = 1.0_f64;
    {
        let mut i: i64 = 0_i64;
        while i < (p.as_ref().unwrap().degree.unwrap() + 1_i64) {
            result = (result + (p.as_ref().unwrap().co[i as usize] * xpow));
            xpow = (xpow * x);
            i += 1_i64;
        }
    }
    return result;
}

pub fn add_poly(mut a: Option<Box<Poly>>, mut b: Option<Box<Poly>>) -> Option<Box<Poly>> {
    let mut deg: i64 = a.as_ref().unwrap().degree.unwrap();
    if (b.as_ref().unwrap().degree.unwrap() > deg) {
        deg = b.as_ref().unwrap().degree.unwrap();
    }
    let mut result: Option<Box<Poly>> = make_poly(deg);
    {
        let mut i: i64 = 0_i64;
        while i < (a.as_ref().unwrap().degree.unwrap() + 1_i64) {
            result.as_ref().unwrap().co[i as usize] = (result.as_ref().unwrap().co[i as usize] + a.as_ref().unwrap().co[i as usize]);
            i += 1_i64;
        }
    }
    {
        let mut i: i64 = 0_i64;
        while i < (b.as_ref().unwrap().degree.unwrap() + 1_i64) {
            result.as_ref().unwrap().co[i as usize] = (result.as_ref().unwrap().co[i as usize] + b.as_ref().unwrap().co[i as usize]);
            i += 1_i64;
        }
    }
    return result;
}

pub fn mul_poly(mut a: Option<Box<Poly>>, mut b: Option<Box<Poly>>) -> Option<Box<Poly>> {
    let mut deg: i64 = (a.as_ref().unwrap().degree.unwrap() + b.as_ref().unwrap().degree.unwrap());
    let mut result: Option<Box<Poly>> = make_poly(deg);
    {
        let mut i: i64 = 0_i64;
        while i < (a.as_ref().unwrap().degree.unwrap() + 1_i64) {
            {
                let mut j: i64 = 0_i64;
                while j < (b.as_ref().unwrap().degree.unwrap() + 1_i64) {
                    result.as_ref().unwrap().co[(i + j) as usize] = (result.as_ref().unwrap().co[(i + j) as usize] + (a.as_ref().unwrap().co[i as usize] * b.as_ref().unwrap().co[j as usize]));
                    j += 1_i64;
                }
            }
            i += 1_i64;
        }
    }
    return result;
}

pub fn derive_poly(mut p: Option<Box<Poly>>) -> Option<Box<Poly>> {
    if (p.as_ref().unwrap().degree.unwrap() == 0_i64) {
        return make_poly(0_i64);
    }
    let mut d: Option<Box<Poly>> = make_poly((p.as_ref().unwrap().degree.unwrap() - 1_i64));
    {
        let mut i: i64 = 1_i64;
        while i < (p.as_ref().unwrap().degree.unwrap() + 1_i64) {
            d.as_ref().unwrap().co[(i - 1_i64) as usize] = ((i as f64) * p.as_ref().unwrap().co[i as usize]);
            i += 1_i64;
        }
    }
    return d;
}

fn main() {
    let mut p1: Option<Box<Poly>> = make_poly(2_i64);
    set_coeff(p1, 0_i64, 1.0_f64);
    set_coeff(p1, 1_i64, 2.0_f64);
    set_coeff(p1, 2_i64, 3.0_f64);
    let mut p2: Option<Box<Poly>> = make_poly(1_i64);
    set_coeff(p2, 0_i64, 1.0_f64);
    set_coeff(p2, 1_i64, (-1.0_f64));
    let mut sum: Option<Box<Poly>> = add_poly(p1, p2);
    let mut product: Option<Box<Poly>> = mul_poly(p1, p2);
    let mut deriv: Option<Box<Poly>> = derive_poly(p1);
    let mut at2: f64 = eval_poly(p1, 2.0_f64);
    let mut dat2: f64 = eval_poly(deriv, 2.0_f64);
    println!("{} {}", "p1(2) = {}".to_string(), at2);
    println!("{} {}", "p1\'(2) = {}".to_string(), dat2);
    println!("{} {}", "(p1*p2)(1) = {}".to_string(), eval_poly(product, 1.0_f64));
}
