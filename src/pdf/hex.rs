pub fn decode_hex_bytes(hex: &[u8]) -> Vec<u8> {
    let mut digits: Vec<u8> = hex
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    if digits.len() % 2 == 1 {
        digits.push(b'0');
    }
    digits
        .chunks(2)
        .filter_map(|chunk| std::str::from_utf8(chunk).ok())
        .filter_map(|byte| u8::from_str_radix(byte, 16).ok())
        .collect()
}
