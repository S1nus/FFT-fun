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
    positive_blinds: Vec<Scalar>,
    negative_blinds: Vec<Scalar>,
    column_blinds: Vec<Scalar>,
    n: usize,
}

pub struct Commitment {
    positive_row_commitments: Vec<Point>,
    negative_row_commitments: Vec<Point>,
    column_blinds_commitment: Point,
}

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

        let mut rng = OsRng::default();

        let n = self.n;

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

        // Generate blinding factors for each row
        let negative_blinds: Vec<Scalar> = (0..n).map(|_| Scalar::random(&mut rng)).collect();
        let positive_blinds: Vec<Scalar> = (0..n).map(|_| Scalar::random(&mut rng)).collect();
        // Generate column blinds
        let column_blinds: Vec<Scalar> = (0..n - 1)
            .map(|_| Scalar::random(&mut rng))
            .chain(iter::once(Scalar::zero()))
            .collect();

        let mut positive_row_commitments: Vec<Ep> = Vec::with_capacity(n);
        let mut negative_row_commitments: Vec<Ep> = Vec::with_capacity(n);

        // commit to the negative rows
        // normally, since the column blinds are just for the positives
        for i in 0..n {
            let mut row_commitment = Point::identity();
            for j in 0..n {
                row_commitment += self.g_s[j] * negative_coefficients[i*n + j];
            }
            row_commitment += self.h * negative_blinds[i];
            negative_row_commitments.push(row_commitment);
        }

        // commit to the first positive row
        // this requires subtracting the column blinds
        let mut first_positive_row_commitment = Point::identity();
        first_positive_row_commitment += self.g_s[0] * positive_coefficients[0];
        for i in 1..n {
            first_positive_row_commitment += self.g_s[i] * (positive_coefficients[i] - column_blinds[i-1]);
        }
        // commit to the rest of the positive rows
        for i in 1..n {
            let mut row_commitment = Point::identity();
            for j in 0..n {
                row_commitment += self.g_s[j] * positive_coefficients[i*n + j];
            }
            row_commitment += self.h * positive_blinds[i];
            positive_row_commitments.push(row_commitment);
        }
        
        // Commit to the column blinds
        let mut column_blinds_commitment = Point::identity();
        for i in 0..n-1 {
            column_blinds_commitment += self.g_s[i] * column_blinds[i];
        }
        column_blinds_commitment += self.g_s[n-1] * Scalar::zero();

        (Commitment{positive_row_commitments, negative_row_commitments, column_blinds_commitment}, CommitmentKey{positive_coefficients, negative_coefficients, positive_blinds, negative_blinds, column_blinds, n})
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