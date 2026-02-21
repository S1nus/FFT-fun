use pasta_curves::{
    Ep,
    group::{Group, ff::{PrimeField, Field}},
    pallas::{Point, Scalar},
    arithmetic::CurveExt,
};
use rand::rngs::OsRng;
use super::Pcs;
use std::iter;

pub struct Bootle16StandardPCS {
    // there will be n points in g_s
    g_s: Vec<Point>,
    // h is one point for the row blind
    h: Point,
    // side length of square
    n: usize,
}

pub struct CommitmentKey {
    // coeffs, padded to n^2
    coefficients: Vec<Scalar>,
    // length n, one for each row
    row_blinds: Vec<Scalar>,
    // length n, the first column in the first row is offset by one, the last column in the last row has zero as its column blind
    // column_blinds[n-1] = zero
    column_blinds: Vec<Scalar>, 
    n: usize,
}

pub struct Commitment {
    // length n
    row_commitments: Vec<Point>, 
    // commitment to the special row ("U"), for the column blinds
    column_blinds_commitment: Point 
}

pub struct Opening {
    x: Scalar,
    y: Scalar,
    t_bar: Vec<Scalar>,  // length n
    tau_bar: Scalar,     // combined blinding factor
}

impl Pcs for Bootle16StandardPCS {
    type Commitment = Commitment;
    type CommitmentKey = CommitmentKey;
    type Opening = Opening;

    fn setup(size: usize) -> Self {
        let n = (size as f64).sqrt().ceil() as usize;
        let hasher = Ep::hash_to_curve("bootle16");
        
        let mut g_s = Vec::with_capacity(n);
        for i in 0..n {
            let g = hasher(&[b'g', i as u8]);
            g_s.push(g);
        }
        let h = hasher(&[b'h']);
        
        Self { g_s, h, n }
    }

    fn commit(&self, coefficients: &[Scalar]) -> (Commitment, CommitmentKey) {
        let mut rng = OsRng::default();
        let n = self.n;
        let n_squared = n * n;
        
        // Pad coefficients to n^2
        let mut coeffs = vec![Scalar::ZERO; n_squared];
        for (i, c) in coefficients.iter().enumerate().take(n_squared) {
            coeffs[i] = *c;
        }
        
        // Generate blinding factors for each row
        let blinds: Vec<Scalar> = (0..n).map(|_| Scalar::random(&mut rng)).collect();
        // Generate column blinds
        let column_blinds: Vec<Scalar> = (0..n - 1)
            .map(|_| Scalar::random(&mut rng))
            .chain(iter::once(Scalar::zero()))
            .collect();
        
        // Commit to each row: T_i = g_1^{t_{i,0}} * g_2^{t_{i,1}} * ... * g_n^{t_{i,n-1}} * h^{τ_i}
        let mut row_commitments = Vec::with_capacity(n);

        // Commit to the first row, which requires us to subtract the column blinds from the highest n-1 columns
        let mut first_row_commitment = Point::identity();
        first_row_commitment +=  self.g_s[0] * coeffs[0];
        for i in 1..n {
            first_row_commitment += self.g_s[i] * (coeffs[i] - column_blinds[i-1]);
        }
        first_row_commitment += self.h * blinds[0];
        row_commitments.push(first_row_commitment);

        // Commit to the rest of the rows, normally
        for i in 1..n {
            let mut row_commitment = Point::identity();
            for j in 0..n {
                row_commitment += self.g_s[j] * coeffs[i * n + j];
            }
            row_commitment += self.h * blinds[i];
            row_commitments.push(row_commitment);
        }

        // Commit to the column blinds
        let mut column_blinds_commitment = Point::identity();
        for i in 0..n-1 {
            column_blinds_commitment += self.g_s[i] * column_blinds[i];
        }
        column_blinds_commitment += self.g_s[n-1] * Scalar::zero();
        
        (
            Commitment { row_commitments, column_blinds_commitment},
            CommitmentKey { coefficients: coeffs, row_blinds: blinds, column_blinds, n}
        )
    }

    fn open(&self, commitment_key: &CommitmentKey, x: Scalar) -> Opening {
        let mut evaluation = Scalar::zero();
        let mut current_x = Scalar::one();
        for coeff in &commitment_key.coefficients {
            evaluation += coeff * current_x;
            current_x *= x;
        }

        let n = commitment_key.n;
        
        // Compute x^n (we'll need powers x^0, x^n, x^{2n}, ..., x^{(n-1)n})
        let x_to_n = x.pow([n as u64]);
        
        // Precompute powers of x^n: [1, x^n, x^{2n}, ..., x^{(n-1)n}]
        let mut x_n_powers = vec![Scalar::ONE; n];
        for i in 1..n {
            x_n_powers[i] = x_n_powers[i - 1] * x_to_n;
        }
        
        // Compute t̄_j = Σ_{i=0}^{n-1} t_{i,j} * x^{in}
        // This is the weighted column sum
        let mut t_bar = vec![Scalar::ZERO; n];
        for j in 0..n {
            for i in 0..n {
                // t_{i,j} is at index i*n + j
                t_bar[j] += commitment_key.coefficients[i * n + j] * x_n_powers[i];
            }
            t_bar[j] += commitment_key.column_blinds[j] * x;
        }

        // Subtract column binds from first row coeffs, same as during commit
        for j in 1..n {
            t_bar[j] -= commitment_key.column_blinds[j-1];
        }
        
        // Compute τ̄ = Σ_{i=0}^{n-1} τ_i * x^{in}
        let mut tau_bar = Scalar::ZERO;
        for i in 0..n {
            tau_bar += commitment_key.row_blinds[i] * x_n_powers[i];
        }

        // Hmm, I don't currently store a blind for the commitment to the column blinds
        // maybe i can leave it empty, and thus "zero", because they are random values themselves
        // for now, i will not add it to tau_bar because it doesn't exist. it implicitly == zero.
        // tau_bar += commitment_key.column_blinds_commitment;
        
        Opening { x, y: evaluation, t_bar, tau_bar }
    }

    fn verify_open(&self, commitment: &Commitment, opening: &Opening) -> bool {
        let n = commitment.row_commitments.len();
        
        // Precompute powers of x^n
        let mut x_n_powers = vec![Scalar::ONE; n];
        for i in 1..n {
            x_n_powers[i] = x_n_powers[i - 1] * opening.x.pow([n as u64]);
        }
        
        // Compute Π_{i=0}^{n-1} T_i^{x^{in}} using homomorphic property
        let mut combined_commitment = Point::identity();
        for i in 0..n {
            combined_commitment += commitment.row_commitments[i] * x_n_powers[i];
        }
        combined_commitment += commitment.column_blinds_commitment * opening.x;
        
        // Compute commitment to t̄ with blinding τ̄
        // Com(t̄; τ̄) = g_1^{t̄_0} * g_2^{t̄_1} * ... * g_n^{t̄_{n-1}} * h^{τ̄}
        let mut expected_commitment = Point::identity();
        for j in 0..n {
            expected_commitment += self.g_s[j] * opening.t_bar[j];
        }
        expected_commitment += self.h * opening.tau_bar;
        
        // Check: Π T_i^{x^{in}} == Com(t̄; τ̄)
        if combined_commitment != expected_commitment {
            return false;
        }

        let mut eval = Scalar::zero();
        let mut current_x = Scalar::one();
        for t in &opening.t_bar {
            eval += t * current_x;
            current_x *= opening.x;
        }
        if eval != opening.y {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_open_verify() {
        let pcs = Bootle16StandardPCS::setup(10);
        let coefficients: Vec<Scalar> = (0..10).map(|i| Scalar::from(i)).collect();
        let (commitment, commitment_key) = pcs.commit(&coefficients);
        let opening = pcs.open(&commitment_key, Scalar::from(5));
        println!("commitment num points: {:?}", commitment.row_commitments.len());
        assert!(pcs.verify_open(&commitment, &opening));
    }
}