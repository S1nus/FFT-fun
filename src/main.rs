use pasta_curves::{
    Ep,
    group::{Group, ff::{PrimeField, Field}}, 
    pallas::{Point, Scalar},
    arithmetic::CurveExt,
};
use rand::rngs::OsRng;
use core::ops::Mul;

mod pedersen;
use pedersen::PedersenPolynomialCommitmentScheme;

fn main() {
}

/*pub trait Pcs {
    fn setup(size: usize) -> Self;
    fn commit(&self, coefficients: &[Scalar]) -> Point;
    fn open(&self, coefficients: &[Scalar], point: Scalar) -> (Point, Vec<Point>);
    fn verify_open(&self, commitment: Point, point: Point, proof: Vec<Point>) -> bool;
}*/