use bitvec::{bitbox, boxed::BitBox, field::BitField};

/// Compute the reduced row echelon form of an augmented binary matrix in place.
pub fn z2_rref(mat: &mut impl AsMut<[BitBox]>) {
    let mat = mat.as_mut();
    if mat.is_empty() {
        return;
    }
    let n = mat[0].len();
    let mut tmp = bitbox![0; n];
    let mut i = 0;
    let mut j = 0;

    while i < n - 1 && j < mat.len() {
        match mat.iter().skip(j).position(|r| r[i]) {
            None => {
                i += 1;
                continue;
            }
            Some(pivot) => mat.swap(j, j + pivot),
        };
        tmp.copy_from_bitslice(&mat[j]);

        for k in 0..mat.len() {
            if k != j && mat[k][i] {
                mat[k] ^= &tmp;
            }
        }

        j += 1;
    }
}

/// Basis of a solution to a system of linear equations in Z2.
#[derive(Debug)]
pub struct Basis {
    pub dim: usize,
    pub vectors: Vec<BitBox>,
}

impl Basis {
    /// Create a basis from a reduced-row-echelon-form matrix in Z2 representing
    /// a solved system of equations.
    pub fn from_rref(rref: &impl AsRef<[BitBox]>) -> Option<Basis> {
        let rref = rref.as_ref();
        if rref.is_empty() {
            return None;
        }

        // Find the free variables
        let mut free_vars = Vec::new();
        let (mut i, mut j) = (0, 0);
        while i < rref.len() && j < rref[i].len() - 1 {
            match rref[i][j] {
                true => {
                    i += 1;
                }
                false => {
                    free_vars.push(j);
                }
            };
            j += 1;
        }

        // Check remaining rows for any unsolvable constraints
        if rref.iter().skip(i).any(|r| *r.last().unwrap()) {
            return None;
        }

        // Go through the rows again and create the basis rows
        let mut vectors = Vec::new();
        let (mut i, mut j) = (0, 0);
        while i < rref.len() && j < rref[i].len() - 1 {
            vectors.push(match rref[i][j] {
                true => {
                    i += 1;
                    free_vars
                        .iter()
                        .map(|&v| rref[i - 1][v])
                        .chain([*rref[i - 1].last().unwrap()])
                        .collect()
                }
                false => free_vars.iter().map(|&v| v == j).chain([false]).collect(),
            });
            j += 1;
        }

        Some(Basis {
            dim: free_vars.len(),
            vectors,
        })
    }

    /// Enumerates all solutions, passing them to the provided callback function.
    /// Does not allocate during enumeration.
    pub fn enumerate(&self, mut callback: impl FnMut(&BitBox)) {
        let mut current_solution = bitbox![0; self.vectors.len()];
        let mut xor_buffer = bitbox![0; self.dim + 1];
        xor_buffer.set(self.dim, true);

        for x in 0..(1usize << self.dim) {
            current_solution.fill_with(|i| {
                xor_buffer[..self.dim].store(x);
                xor_buffer.set(self.dim, self.vectors[i][self.dim]);
                xor_buffer &= &self.vectors[i];
                xor_buffer.count_ones() % 2 != 0
            });
            callback(&current_solution);
        }
    }
}
