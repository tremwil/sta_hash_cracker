use bitvec::boxed::BitBox;

/// Compute Shadow Tower Abyss file hash for a given byte slice.
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

/// Safe transmute from a bitbox to a byte slice.
///
/// # Panics
/// If `bits.len()` is not a multiple of 8.
pub fn bits_as_byte_slice(bits: &BitBox) -> &[u8] {
    assert!(bits.len() % 8 == 0);
    unsafe { std::slice::from_raw_parts(bits.as_raw_slice().as_ptr() as *const u8, bits.len() / 8) }
}
