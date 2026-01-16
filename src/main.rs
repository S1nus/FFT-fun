use pasta_curves::{
    Ep,
    group::{Group, ff::{PrimeField, Field}}, 
    pallas::{Point, Scalar},
    arithmetic::CurveExt,
};
use rand::rngs::OsRng;
use core::ops::Mul;
use std::collections::HashSet;

fn main() {
    let modulus: u64 = 337;
    let root_of_unity: u64 = 85;

    let mut unique_elements = HashSet::new();
    for i in 0..modulus {
        let element = mod_exp(root_of_unity, i, modulus);
        unique_elements.insert(element);
    }
    println!("num unique elements: {}", unique_elements.len());

    let domain_size: u64 = unique_elements.len() as u64;

    let mut unique_squares = HashSet::new();
    for i in 0..domain_size {
        let element = mod_exp(root_of_unity, i, modulus);
        let square = mod_exp(element, 2, modulus);
        unique_squares.insert(square);
        println!("square: {}", square);
    }
    println!("Num unique squares: {}", unique_squares.len());
}

fn mod_exp(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result = 1;
    base %= modulus;
    while exp > 0 {
        if exp % 2 == 1 {
            result = result * base % modulus;
        }
        exp /= 2;
        base = base * base % modulus;
    }
    result
}
