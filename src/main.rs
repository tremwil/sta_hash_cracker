use std::{fs::{File, OpenOptions}, io::{BufWriter, Write}};

use bitvec::{bitbox, boxed::BitBox, field::BitField};

pub fn sta_hash(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0;
    for c in bytes {
        h = h.rotate_left(6) ^ (*c as u32)
    }
    h
}

pub fn print_mat(m: impl AsRef<[BitBox]>) {
    for row in m.as_ref() {
        println!("{}", row)
    }
}

/// Linear equations for `sta_hash(vec) == hash`
pub fn hash_constraints(n: usize, hash: u32) -> Vec<BitBox> {
    let mut mat: Vec<_> = (0..32).map(|i| {
        let mut row = bitbox![0; 8*n +1];
        row.last_mut().unwrap().set(hash & (1 << i) != 0);
        row
    }).collect();

    for i in 0..n {
        for j in 0..8 {
            let hash_pos = (6*(n - i - 1) + j) % 32;
            mat[hash_pos].get_mut(8*i+j).map(|mut r| *r ^= true);
        }
    }

    mat
}

/// Linear equations for all characters being within the 32-96 range
pub fn range_constraints(n: usize) -> Vec<BitBox> {
    let top_bit_zero = (0..n).map(move |i| {
        let mut row = bitbox![0; 8*n+1];
        row.set(8*i + 7, true);
        row
    });

    let bits_5_and_6_exclusive = (0..n).map(move |i| {
        let mut row = bitbox![0; 8*n+1];
        row[8*i + 5 .. 8*i + 7].store(0b11);
        row.set(8*n, true);
        row
    });

    top_bit_zero.chain(bits_5_and_6_exclusive).collect()
}

/// Linear equations for a specific character being an exact value
pub fn char_constraint(n: usize, idx: usize, value: u8) -> Vec<BitBox> {
    (0..8).map(|i| {
        let mut row = bitbox![0; 8*n +1];
        row.set(8*idx + i, true);
        row.last_mut().unwrap().set(value & (1 << i) != 0);
        row
    }).collect()
}

/// Compute the reduced row echelon form of an augmented matrix in Z2.
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
            None => { i += 1; continue; },
            Some(pivot) => mat.swap(j, j + pivot)
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

#[derive(Debug)]
pub struct Basis {
    dim: usize,
    vectors: Vec<BitBox>
}

pub fn solution_basis(rref: &impl AsRef<[BitBox]>) -> Option<Basis> {
    let rref = rref.as_ref();
    if rref.is_empty() {
        return None;
    }
    
    // Find the free variables
    let mut free_vars = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < rref.len() && j < rref[i].len() - 1 {
        match rref[i][j] {
            true => { i += 1; },
            false => { free_vars.push(j); }
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
                free_vars.iter()
                    .map(|&v| rref[i - 1][v])
                    .chain([*rref[i - 1].last().unwrap()])
                    .collect()
            },
            false => free_vars.iter()
                .map(|&v| v == j)
                .chain([false])
                .collect()
        }); 
        j += 1;
    }

    Some(Basis {
        dim: free_vars.len(),
        vectors
    })
}

pub fn generate_solutions(basis: &Basis, mut callback: impl FnMut(&BitBox)) {
    let mut current_solution = bitbox![0; basis.vectors.len()];
    let mut xor_buffer = bitbox![0; basis.dim + 1];
    xor_buffer.set(basis.dim, true);

    for x in 0..(1usize << basis.dim) {
        current_solution.fill_with(|i| {
            xor_buffer[..basis.dim].store(x);
            xor_buffer.set(basis.dim, basis.vectors[i][basis.dim]);
            xor_buffer &= &basis.vectors[i];
            xor_buffer.count_ones() % 2 != 0
        });
        callback(&current_solution);
    }
}

pub fn bits_as_byte_slice(bits: &BitBox) -> &[u8] {
    assert!(bits.len() % 8 == 0);
    unsafe {
        std::slice::from_raw_parts(
            bits.as_raw_slice().as_ptr() as *const u8,
            bits.len() / 8
        )
    }
}

fn main() {
    let n = 8;
    let hash = 0x52bc6ce4;

    let mut mat = Vec::new();
    mat.extend(hash_constraints(n, hash));
    mat.extend(range_constraints(n));

    z2_rref(&mut mat);
    if let Some(basis) = solution_basis(&mat) {
        println!("Found {} solutions", 1usize << basis.dim);

        let mut file = BufWriter::new(
            OpenOptions::new().write(true).truncate(true)
                .open("matches.txt").unwrap()
        );
        generate_solutions(&basis, |s| {
           file.write(bits_as_byte_slice(s)).unwrap();
           file.write(b"\n").unwrap();
        });

        file.flush().unwrap();
    }

    // const LEN: usize = 9;
    // let h = 0x52bc6ce4; // sta_hash(b"HELP.TXM");
    // println!("target hash: {:x}", h);

    // let p = PartialSolve::new(h, LEN);

    // fn search(p: &PartialSolve, output: &mut Vec<BitBox>) {
    //     if let Some(i) = p.free_vars.first_one() {
    //         for v in 0..4 {
    //             let mut with_value = p.clone();
    //             with_value.bind(i, v);
    //             if !with_value.is_candidate() {
    //                 continue;
    //             }
    //             search(&with_value, output);
    //         }
    //     }
    //     else {
    //         output.push(p.solution.clone());
    //     }
    // }

    // let t = Instant::now();

    // let mut solutions = Vec::new();
    // search(&p, &mut solutions);

    // println!("Found {} matches in {:?}", solutions.len(), t.elapsed());
    // let filtered: Vec<_> = solutions.into_iter().filter_map(|bits| {
    //     let bytes = unsafe {
    //         std::slice::from_raw_parts(
    //             bits.as_raw_slice() as *const _ as *const u8, 
    //             bits.len() / 8
    //         )
    //     };

    //     if bytes.iter().all(|&c| c.is_ascii_alphanumeric() || c == b'_' || c == b'.')
    //         && bytes.first().map(|b| b.is_ascii_alphabetic()) == Some(true) {
    //         Some(String::from_utf8_lossy(bytes).to_string())
    //     }
    //     else {
    //         None
    //     }
    // }).collect();

    // println!("Filtered to {} matches and dumped to matches.txt", filtered.len());
    // std::fs::write("matches.txt", filtered.join("\n")).unwrap();
}
