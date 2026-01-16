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
    // 8th root of unity (2^3th root of unity)
    let root_of_unity: u64 = 85;

    let domain: Vec<u64> = (0..8).map(|i| mod_exp(root_of_unity, i, modulus)).collect();

    let polynomial: Vec<u64> = vec![3,1,4,1,5,9,2,6];

    let result = evaluate_polynomial_naive(polynomial.clone(), domain.clone(), modulus);
    let result_fft = evaluate_polynomial_fft(polynomial.clone(), domain.clone(), modulus);
}

fn evaluate_polynomial_naive(polynomial: Vec<u64>, domain: Vec<u64>, modulus: u64) -> Vec<u64> {
    println!("Evaluating polynomial of size {} (naive) at {} points", polynomial.len(), domain.len());

    let mut num_mod_exps = 0;
    let mut num_adds = 0;
    let mut num_muls = 0;

    let mut result = Vec::new();
    for x in domain {
        let mut value: u64 = 0;
        for (i, coeff) in polynomial.iter().enumerate() {
            value = (value + coeff * mod_exp(x, i as u64, modulus)) % modulus;
            num_mod_exps += 1;
            num_muls += 1;
            num_adds += 1;
        }
    }
    println!("exps: {}, adds: {}, muls: {}", num_mod_exps, num_adds, num_muls);
    result
}

fn evaluate_polynomial_fft(polynomial: Vec<u64>, domain: Vec<u64>, modulus: u64) -> Vec<u64> {
    println!("Evaluating polynomial of size {} (fft) at {} points", polynomial.len(), domain.len());

    let mut num_mod_exps = 0;
    let mut num_adds = 0;
    let mut num_muls = 0;

    if polynomial.len() == 1 {
    }

    let mut evens = polynomial.iter().step_by(2).map(|x| *x).collect::<Vec<u64>>();
    let mut odds = polynomial.iter().skip(1).step_by(2).map(|x| *x).collect::<Vec<u64>>();

    let halved_domain;
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
