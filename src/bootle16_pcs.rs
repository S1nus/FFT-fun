use pasta_curves::{
    Ep,
    group::{Group, ff::{PrimeField, Field}},
    pallas::{Point, Scalar},
    arithmetic::CurveExt,
};
use rand::rngs::OsRng;
use super::Pcs;

pub struct Bootle16PCS {
    g_s: Vec<Point>,
    h: Point,
    n: usize, // side length of square
}

pub struct CommitmentKey {
    coefficients: Vec<Scalar>, // padded to n^2
    blinds: Vec<Scalar>,       // length n
    column_blinds: Vec<Scalar>, // length n - 1
    n: usize,
}

pub struct Commitment {
    row_commitments: Vec<Point>, // length n
}

pub struct Opening {
    x: Scalar,
    t_bar: Vec<Scalar>,  // length n
    tau_bar: Scalar,     // combined blinding factor
}

impl Pcs for Bootle16PCS {
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
        let column_blinds: Vec<Scalar> = (0..n - 1).map(|_| Scalar::random(&mut rng)).collect();
        
        // Commit to each row: T_i = g_1^{t_{i,0}} * g_2^{t_{i,1}} * ... * g_n^{t_{i,n-1}} * h^{τ_i}
        let mut row_commitments = Vec::with_capacity(n);
        for i in 0..n {
            let mut row_commitment = Point::identity();
            for j in 0..n {
                row_commitment += self.g_s[j] * coeffs[i * n + j];
            }
            row_commitment += self.h * blinds[i];
            row_commitments.push(row_commitment);
        }
        
        (
            Commitment { row_commitments },
            CommitmentKey { coefficients: coeffs, blinds, column_blinds, n }
        )
    }

    fn open(&self, commitment_key: &CommitmentKey, x: Scalar) -> Opening {
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
        }
        
        // Compute τ̄ = Σ_{i=0}^{n-1} τ_i * x^{in}
        let mut tau_bar = Scalar::ZERO;
        for i in 0..n {
            tau_bar += commitment_key.blinds[i] * x_n_powers[i];
        }
        
        Opening { x, t_bar, tau_bar }
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
        
        // Compute commitment to t̄ with blinding τ̄
        // Com(t̄; τ̄) = g_1^{t̄_0} * g_2^{t̄_1} * ... * g_n^{t̄_{n-1}} * h^{τ̄}
        let mut expected_commitment = Point::identity();
        for j in 0..n {
            expected_commitment += self.g_s[j] * opening.t_bar[j];
        }
        expected_commitment += self.h * opening.tau_bar;
        
        // Check: Π T_i^{x^{in}} == Com(t̄; τ̄)
        combined_commitment == expected_commitment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_open_verify() {
        let pcs = Bootle16PCS::setup(10);
        let coefficients: Vec<Scalar> = (0..10).map(|i| Scalar::from(i)).collect();
        let (commitment, commitment_key) = pcs.commit(&coefficients);
        let opening = pcs.open(&commitment_key, Scalar::from(5));
        println!("commitment num points: {:?}", commitment.row_commitments.len());
        assert!(pcs.verify_open(&commitment, &opening));
    }
}