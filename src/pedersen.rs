use pasta_curves::{
    Ep,
    group::{Group, ff::{PrimeField, Field}}, 
    pallas::{Point, Scalar},
    arithmetic::CurveExt,
};
use rand::rngs::OsRng;
use core::ops::Mul;

pub struct PedersenPolynomialCommitmentScheme {
    g: Point,
    h: Point,
}

pub struct CommitmentKey {
    coefficients: Vec<Scalar>,
    blinds: Vec<Scalar>,
}

pub struct Commitment {
    points: Vec<Point>,
}

pub struct Opening {
    x: Scalar,
    y: Scalar,
    r_bar: Scalar,
}

impl PedersenPolynomialCommitmentScheme {
    pub fn setup() -> Self {
        let hasher = Ep::hash_to_curve("cnode");
        let g = hasher(&[b'g']);
        let h = hasher(&[b'h']);
        Self { g, h }
    }

    pub fn commit(&self, coefficients: &[Scalar]) -> (Commitment, CommitmentKey) {
        let mut rng = OsRng::default();
        let mut blinds = Vec::with_capacity(coefficients.len());
        let mut commitment_points = Vec::with_capacity(coefficients.len());

        for i in 0..coefficients.len() {
            blinds.push(Scalar::random(&mut rng));
            commitment_points.push(self.g * coefficients[i] + self.h * blinds[i]);
        }
        (Commitment{points: commitment_points}, CommitmentKey{coefficients: coefficients.to_vec(), blinds})
    }

    pub fn open(&self, commitment_key: &CommitmentKey, x: Scalar) -> Opening {
        let mut y = Scalar::ZERO;
        let mut r_bar = Scalar::ZERO;
        for i in 0..commitment_key.coefficients.len() {
            y += commitment_key.coefficients[i] * x.pow([i as u64]);
            r_bar += commitment_key.blinds[i] * x.pow([i as u64]);
        }
        Opening { x, y, r_bar }
    }

    pub fn verify_open(&self, commitment: &Commitment, opening: &Opening) -> bool {
        let mut c = commitment.points[0];
        for i in 1..commitment.points.len() {
            c += commitment.points[i] * opening.x.pow([i as u64]);
        }
        c == self.g * opening.y + self.h * opening.r_bar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_open_verify() {
        let pcs = PedersenPolynomialCommitmentScheme::setup();
        let coefficients: Vec<Scalar> = (0..10).map(|i| Scalar::from(i)).collect();
        let (commitment, commitment_key) = pcs.commit(&coefficients);
        let opening = pcs.open(&commitment_key, Scalar::from(5));
        assert!(pcs.verify_open(&commitment, &opening));
    }
}