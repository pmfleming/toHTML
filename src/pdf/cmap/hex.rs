pub fn code_range(start: &[u8], end: &[u8]) -> Option<(u32, u32)> {
    Some((code_value(start)?, code_value(end)?))
}

pub fn code_value(bytes: &[u8]) -> Option<u32> {
    bytes.iter().try_fold(0u32, |value, byte| {
        value.checked_mul(256)?.checked_add(u32::from(*byte))
    })
}

pub fn code_bytes(value: u32, reference: &[u8]) -> Vec<u8> {
    let len = reference.len().max(1);
    (0..len)
        .rev()
        .map(|shift| ((value >> (shift * 8)) & 0xff) as u8)
        .collect()
}

pub fn hex_bytes_lossy(hex: &[u8]) -> Vec<u8> {
    super::super::hex::decode_hex_bytes(hex)
}
