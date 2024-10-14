use bitvec::{bitbox, boxed::BitBox, field::BitField};

/// system of equations for having a specific sta_hash value
pub fn sta_hash(n: usize, hash: u32) -> Vec<BitBox> {
    let mut mat: Vec<_> = (0..32)
        .map(|i| {
            let mut row = bitbox![0; 8*n +1];
            row.last_mut().unwrap().set(hash & (1 << i) != 0);
            row
        })
        .collect();

    for i in 0..n {
        for j in 0..8 {
            let hash_pos = (6 * (n - i - 1) + j) % 32;
            mat[hash_pos].get_mut(8 * i + j).map(|mut r| *r ^= true);
        }
    }

    mat
}

/// system of equations for the given characters being within the 32-96 range
pub fn approx_uppercase_alphanumeric(
    n: usize,
    indices: impl IntoIterator<Item = usize> + Clone,
) -> Vec<BitBox> {
    let top_bit_zero = indices.clone().into_iter().map(move |i| {
        let mut row = bitbox![0; 8*n+1];
        row.set(8 * i + 7, true);
        row
    });

    let bits_5_and_6_exclusive = indices.into_iter().map(move |i| {
        let mut row = bitbox![0; 8*n+1];
        row[8 * i + 5..8 * i + 7].store(0b11);
        row.set(8 * n, true);
        row
    });

    top_bit_zero.chain(bits_5_and_6_exclusive).collect()
}

// system of equations for the given characters being in the 64-96 range
pub fn approx_uppercase_alphabetic(
    n: usize,
    indices: impl IntoIterator<Item = usize> + Clone,
) -> Vec<BitBox> {
    let mut eqs = approx_uppercase_alphanumeric(n, indices.clone());

    let bit_6_set = indices.into_iter().map(move |i| {
        let mut row = bitbox![0; 8*n+1];
        row.set(8 * i + 6, true);
        row.set(8 * n, true);
        row
    });

    eqs.extend(bit_6_set);
    eqs
}

/// Linear equations for a specific byte being an exact value
pub fn byte(n: usize, idx: usize, value: u8) -> Vec<BitBox> {
    (0..8)
        .map(|i| {
            let mut row = bitbox![0; 8*n +1];
            row.set(8 * idx + i, true);
            row.last_mut().unwrap().set(value & (1 << i) != 0);
            row
        })
        .collect()
}
