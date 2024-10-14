use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{BufWriter, Write},
};

mod constraints;
mod linalg;
mod utils;

fn main() {
    let n = 10;
    let ext_len = 3;
    let hash = 0x45e87010;

    let mut mat: Vec<_> = [
        // Match the hash
        constraints::sta_hash(n, hash),
        // Printable uppercase range
        constraints::approx_uppercase_alphanumeric(n, 0..n),
        // First character is an uppercase letter
        constraints::approx_uppercase_alphabetic(n, 0..1),
        // Assume 3 character long uppercase extension
        constraints::byte(n, n - ext_len - 1, b'.'),
        constraints::approx_uppercase_alphabetic(n, n - ext_len..n),
    ]
    .into_iter()
    .flatten()
    .collect();

    let all_words = std::fs::read("words_alpha.txt").unwrap();
    let words: HashSet<_> = all_words
        .split(|&p| p == b'\n')
        .filter(|w| w.len() >= 3 && w.len() < n - ext_len)
        .collect();

    linalg::z2_rref(&mut mat);
    if let Some(basis) = linalg::Basis::from_rref(&mat) {
        println!("Found {} solutions for {n} characters", 1usize << basis.dim);

        let mut file = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open("matches.txt")
                .unwrap(),
        );

        let mut filtered_count = 0;
        basis.enumerate(|s| {
            let bytes = utils::bits_as_byte_slice(s);
            if !bytes.iter().all(|&b| {
                (b >= b'0' && b <= b'9') || (b >= b'A' && b <= b'Z') || b == b'_' || b == b'.'
            }) {
                return;
            }

            for i in 0..(n - ext_len - 4) {
                for j in (i + 3)..(n - ext_len) {
                    if words.contains(&bytes[i..j]) {
                        filtered_count += 1;
                        file.write(bytes).unwrap();
                        file.write(b"\n").unwrap();
                        return;
                    }
                }
            }
        });

        println!("Filtered to {filtered_count} solutions");
        file.flush().unwrap();
    }
    else {
        println!("No solutions exist for {n} characters");
    }
}
