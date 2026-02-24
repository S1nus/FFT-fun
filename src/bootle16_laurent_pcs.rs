use pasta_curves::{
    Ep,
    group::{Group, ff::{PrimeField, Field}},
    pallas::{Point, Scalar},
    arithmetic::CurveExt,
};
use rand::rngs::OsRng;
use super::Pcs;
use std::iter;

// Only works for Laurent polynomials with a zero constant term
pub struct Bootle16LaurentPCS {
    g_s: Vec<Point>,
    h: Point,
    // width of the rectangle
    n: usize,
    // the length of the negative side of the rectangle
    m1: usize,
    // the length of the positive side of the rectangle
    m2: usize,
}

pub struct CommitmentKey {
    positive_coefficients: Vec<Scalar>,
    negative_coefficients: Vec<Scalar>,
}

pub struct Commitment {}

pub struct Opening {}

impl Pcs for Bootle16LaurentPCS {
    type Commitment = Commitment;
    type CommitmentKey = CommitmentKey;
    type Opening = Opening;

    fn setup(size: usize) -> Self {
        let n = (size as f64).sqrt().ceil() as usize;
        // For simplicity we will require m1 == m2 == n, but this doesn't have to be the case
        let m1 = n;
        let m2 = n;
        let hasher = Ep::hash_to_curve("bootle16");
        
        let mut g_s = Vec::with_capacity(n);
        for i in 0..n {
            let g = hasher(&[b'g', i as u8]);
            g_s.push(g);
        }
        let h = hasher(&[b'h']);
        
        Self { g_s, h, n, m1, m2 }
    }

    fn commit(&self, coefficients: &[Scalar]) -> (Commitment, CommitmentKey) {

        if coefficients.len() % 2 != 0 { panic!("Must have even num coeffs");}

        let negative_coefficients: Vec<Scalar> = coefficients
            .iter()
            .take(coefficients.len() / 2)
            .map(|c| c.clone())
            .collect();
        let positive_coefficients: Vec<Scalar> = coefficients
            .iter()
            .skip(coefficients.len() / 2)
            .map(|c| c.clone())
            .collect();

        let n_squared = self.n * self.n;
        if negative_coefficients.len() != n_squared || positive_coefficients.len() != n_squared {
            panic!("must equal n squared")
        }

        (Commitment{}, CommitmentKey{positive_coefficients, negative_coefficients})
    }

    fn open(&self, commitment_key: &CommitmentKey, x: Scalar) -> Opening {
        Opening {}
    }

    fn verify_open(&self, commitment: &Self::Commitment, opening: &Self::Opening) -> bool {
        false
    }
}

#[cfg(test)]
mod test {
    use crate::bootle16_laurent_pcs::Bootle16LaurentPCS;
    use crate::Pcs;
    use pasta_curves::pallas::Scalar;

    #[test]
    fn test_commit() {
        let pcs = Bootle16LaurentPCS::setup(25);
        let coeffs: Vec<Scalar> = (0..50).map(|i| Scalar::from(i as u64)).collect();
        let (_, ckey) = pcs.commit(&coeffs);
        println!("{} , {}", ckey.positive_coefficients.len(), ckey.negative_coefficients.len());
    }
}