use pasta_curves::{
    Ep,
    group::{Group, ff::{PrimeField, Field}}, 
    pallas::{Point, Scalar},
    arithmetic::CurveExt,
};
use rand::rngs::OsRng;
use core::ops::Mul;

mod pedersen_pcs;
use pedersen_pcs::PedersenPolynomialCommitmentScheme;

mod bootle16_pcs;
use bootle16_pcs::Bootle16PCS;

fn main() {
}

/*pub trait Pcs {
    fn setup(size: usize) -> Self;
    fn commit(&self, coefficients: &[Scalar]) -> Point;
    fn open(&self, coefficients: &[Scalar], point: Scalar) -> (Point, Vec<Point>);
    fn verify_open(&self, commitment: Point, point: Point, proof: Vec<Point>) -> bool;
}*/

pub trait Pcs {
    type Commitment;
    type CommitmentKey;
    type Opening;

    fn setup(size: usize) -> Self;
    fn commit(&self, coefficients: &[Scalar]) -> (Self::Commitment, Self::CommitmentKey);
    fn open(&self, commitment_key: &Self::CommitmentKey, x: Scalar) -> Self::Opening;
    fn verify_open(&self, commitment: &Self::Commitment, opening: &Self::Opening) -> bool;
}