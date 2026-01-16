use pasta_curves::{
    Ep,
    group::{Group, ff::{PrimeField, Field}}, 
    pallas::{Point, Scalar},
    arithmetic::CurveExt,
};
use rand::rngs::OsRng;
use core::ops::Mul;

fn main() {
    // 4 + 3x + 2x^2 + x^3 + x^4
    let my_polynomial = vec![4, 3, 2, 1, 1];
    let poly_num_coeffs = my_polynomial.len();
    let generator = Point::generator();
    let hasher = Ep::hash_to_curve("cnode");
    let g_points: Vec<Ep> = (0..poly_num_coeffs)
        .map(|i| {
            let mut input = Vec::with_capacity(1 + 8);
            input.push(b'g');
            input.extend_from_slice(&i.to_le_bytes());
            hasher(&input)
        })
        .collect();

}