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

    let result = evaluate_polynomial_naive(&polynomial, &domain, modulus);
    let result_one_split = evaluate_polynomial_with_one_split(&polynomial, &domain, modulus);
    let result_fft = evaluate_polynomial_fft(&polynomial, &domain, modulus);
    println!("result: {:?}", result);
    println!("result_one_split: {:?}", result_one_split);
    println!("result_fft: {:?}", result_fft);

    let ifft_result = ifft(&result_fft, &domain, modulus);
    println!("Expected result: {:?}", polynomial);
    println!("ifft_result: {:?}", ifft_result);
}

fn evaluate_polynomial_naive(polynomial: &Vec<u64>, domain: &Vec<u64>, modulus: u64) -> Vec<u64> {
    println!("Evaluating polynomial of size {} (naive) at {} points", polynomial.len(), domain.len());

    let mut num_mod_exps = 0;
    let mut num_adds = 0;
    let mut num_muls = 0;

    let mut result = Vec::new();
    for x in domain {
        let mut value: u64 = 0;
        for (i, coeff) in polynomial.iter().enumerate() {
            value = (value + coeff * mod_exp(*x, i as u64, modulus)) % modulus;
            num_mod_exps += 1;
            num_muls += 1;
            num_adds += 1;
        }
        result.push(value);
    }
    println!("exps: {}, adds: {}, muls: {}", num_mod_exps, num_adds, num_muls);
    result
}

fn evaluate_polynomial_with_one_split(polynomial: &Vec<u64>, domain: &Vec<u64>, modulus: u64) -> Vec<u64> {
    println!("Evaluating polynomial of size {} (fft) at {} points", polynomial.len(), domain.len());

    let mut num_mod_exps = 0;
    let mut num_adds = 0;
    let mut num_muls = 0;

    let mut evens = polynomial.iter().step_by(2).map(|x| *x).collect::<Vec<u64>>();
    let mut odds = polynomial.iter().skip(1).step_by(2).map(|x| *x).collect::<Vec<u64>>();

    let halved_domain = domain[0..domain.len()/2].iter().map(|x| mod_exp(*x, 2, modulus)).collect::<Vec<u64>>();
    println!("halved_domain: {:?}", halved_domain);
    let evens_result = evaluate_polynomial_naive(&evens, &halved_domain, modulus);
    let odds_result = evaluate_polynomial_naive(&odds, &halved_domain, modulus);
    let p_x_results: Vec<u64> = evens_result
        .iter()
        .zip(odds_result.iter())
        .zip(domain.iter())
        .map(|((e, o), x)| 
            (e + (x * o)%modulus) % modulus)
        .collect();

    let p_neg_x_results: Vec<u64> = evens_result
        .iter()
        .zip(odds_result.iter())
        .zip(domain.iter())
        .map(|((e, o), x)| 
            (e + modulus - (x * o) % modulus) % modulus)
        .collect();

    p_x_results.iter().chain(p_neg_x_results.iter()).map(|x| *x).collect()
}

fn evaluate_polynomial_fft(polynomial: &Vec<u64>, domain: &Vec<u64>, modulus: u64) -> Vec<u64> {

    let mut num_mod_exps = 0;
    let mut num_adds = 0;
    let mut num_muls = 0;

    if polynomial.len() == 1 {
        return vec![polynomial[0]];
    }

    let mut evens = polynomial.iter().step_by(2).map(|x| *x).collect::<Vec<u64>>();
    let mut odds = polynomial.iter().skip(1).step_by(2).map(|x| *x).collect::<Vec<u64>>();
    let halved_domain = domain[0..domain.len()/2].iter().map(|x| mod_exp(*x, 2, modulus)).collect::<Vec<u64>>();
    let evens_result = evaluate_polynomial_fft(&evens, &halved_domain, modulus);
    let odds_result = evaluate_polynomial_fft(&odds, &halved_domain, modulus);
    let p_x_results: Vec<u64> = evens_result
        .iter()
        .zip(odds_result.iter())
        .zip(domain.iter())
        .map(|((e, o), x)| 
            (e + (x * o)%modulus) % modulus)
        .collect();
    let p_neg_x_results: Vec<u64> = evens_result
        .iter()
        .zip(odds_result.iter())
        .zip(domain.iter())
        .map(|((e, o), x)| 
            (e + modulus - (x * o) % modulus) % modulus)
        .collect();
    p_x_results.iter().chain(p_neg_x_results.iter()).map(|x| *x).collect()
}

fn ifft(values: &Vec<u64>, domain: &Vec<u64>,modulus: u64) -> Vec<u64> {
    let mut fft_result = evaluate_polynomial_fft(&values, &domain, modulus);
    fft_result[1..].reverse();
    for i in 0..fft_result.len() {
        fft_result[i] = fft_result[i] * mod_exp(domain.len() as u64, modulus - 2, modulus) % modulus;
    }
    fft_result
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
