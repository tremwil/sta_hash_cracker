use std::io::Read;
use std::time::Instant;
use std::{fmt::Write};
use bitvec::field::BitField;
use bitvec::prelude::Lsb0;
use bitvec::slice::BitSlice;
use bitvec::{bitbox, boxed::BitBox, BitArr};

pub fn sta_hash(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0;
    for c in bytes {
        h = h.rotate_left(6) ^ (*c as u32)
    }
    h
}

pub fn print_mat<const N: usize>(m: &[BitBox; N]) {
    for row in m {
        println!("{}", row)
    }
}

pub fn rref(n: usize) -> [BitBox; 16] {
    let mut mat: [_; 16] = std::array::from_fn(|_| bitbox![0; 4*n]);

    for i in 0..n {
        for j in 0..4 {
            let hash_pos = (3*(n - i - 1) + j) % 16;
            mat[hash_pos].get_mut(4*i+j).map(|mut r| *r ^= true);
        }
    }

    mat
}

#[derive(Debug, Clone)]
struct RowBinding {
    terms: BitBox,
    var: usize,
    scalar: usize,
}

#[derive(Debug, Clone)]
struct PartialSolve {
    free_vars: BitBox,
    bindings: Vec<RowBinding>,
    solution: BitBox,
    solved: BitBox,
    test_buffer: BitBox
}

impl PartialSolve {
    pub fn new(hash: u32, n: usize) -> Self {
        let mut bindings = Vec::new();
        let mut free_vars = bitbox![1; 4*n];
        let mut solution = bitbox![0; 8*n];
        let mut solved = bitbox![0; 8*n];
        let test_buffer = bitbox![0; 8*n];
    
        let solution_matrix = rref(n);
        for (i, row) in solution_matrix.iter().enumerate() {
            if let Some(bound) = row.first_one() {
                let mut terms = row.clone();
                free_vars.set(bound, false);
                terms.set(bound, false);

                let scalar = ((hash >> (2 * i)) % 4) as usize;
                if terms.not_any() {
                    solution[2*bound .. 2*(bound+1)].store(scalar);
                    solved[2*bound .. 2*(bound+1)].store(3usize);
                }

                bindings.push(RowBinding {
                    var: bound,
                    terms,
                    scalar
                })
            }
        }
    
        PartialSolve {
            free_vars,
            bindings,
            solution,
            solved,
            test_buffer
        }
    }

    pub fn bind(&mut self, var: usize, value: usize) {
        self.free_vars.set(var, false);

        for b in &mut self.bindings {
            let mut m = b.terms.get_mut(var).unwrap();
            if *m {
                b.scalar ^= value;
                *m = false;

                std::mem::drop(m);
                if b.terms.not_any() {
                    self.solution[2*b.var .. 2*(b.var + 1)].store(b.scalar);
                    self.solved[2*b.var .. 2*(b.var + 1)].store(3usize);
                }
            };
        }

        self.solution[2*var .. 2*(var + 1)].store(value);
        self.solved[2*var .. 2*(var + 1)].store(3usize);
    }

    pub fn is_candidate(&mut self) -> bool {
        // We want to check that the solution chars are within the 32-95 range.
        // Need two checks for this: 
        // - bit 7 is 0
        // - bits 5 and 6 are mutually exclusive (e.g xor to 1)
        // For the first, a simple AND works
        // For the second:
        //  1. apply mask for those bits
        //  2. count bits, expect 

        //println!("{}", &self.solution);
        
        if !self.solution.as_raw_slice().iter().all(|b| b & 0x8080_8080_8080_8080 == 0) {
            return false;
        }

        self.test_buffer.copy_from_bitslice(&self.solution);
        self.test_buffer[..self.solution.len()] ^= &self.solution[1..];

        let ready_mask = self.solved.as_raw_slice().iter();
        return self.test_buffer.as_raw_slice().iter().zip(ready_mask).all(
            |(b, r)| {
                let m = 0x2020_2020_2020_2020 & r & (r >> 1);
                //println!("{:064b} vs {:064b}", b, m);
                b & m == m
            }
        )
    }
}

fn main() {
    const LEN: usize = 9;
    let h = 0x52bc6ce4; // sta_hash(b"HELP.TXM");
    println!("target hash: {:x}", h);

    let p = PartialSolve::new(h, LEN);

    fn search(p: &PartialSolve, output: &mut Vec<BitBox>) {
        if let Some(i) = p.free_vars.first_one() {
            for v in 0..4 {
                let mut with_value = p.clone();
                with_value.bind(i, v);
                if !with_value.is_candidate() {
                    continue;
                }
                search(&with_value, output);
            }
        }
        else {
            output.push(p.solution.clone());
        }
    }

    let t = Instant::now();

    let mut solutions = Vec::new();
    search(&p, &mut solutions);

    println!("Found {} matches in {:?}", solutions.len(), t.elapsed());
    let filtered: Vec<_> = solutions.into_iter().filter_map(|bits| {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                bits.as_raw_slice() as *const _ as *const u8, 
                bits.len() / 8
            )
        };

        if bytes.iter().all(|&c| c.is_ascii_alphanumeric() || c == b'_' || c == b'.')
            && bytes.first().map(|b| b.is_ascii_alphabetic()) == Some(true) {
            Some(String::from_utf8_lossy(bytes).to_string())
        }
        else {
            None
        }
    }).collect();

    println!("Filtered to {} matches and dumped to matches.txt", filtered.len());
    std::fs::write("matches.txt", filtered.join("\n")).unwrap();
}
