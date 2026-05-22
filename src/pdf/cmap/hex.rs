pub fn hex_tokens(line: &str) -> Vec<Vec<u8>> {
    let mut tokens = Vec::new();
    let mut rest = line;

    while let Some((token, next)) = next_hex_token(rest) {
        tokens.push(token);
        rest = next;
    }

    tokens
}

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

fn next_hex_token(rest: &str) -> Option<(Vec<u8>, &str)> {
    let start = rest.find('<')?;
    let after_start = &rest[start + 1..];
    if after_start.starts_with('<') {
        return next_hex_token(after_start);
    }

    let end = after_start.find('>')?;
    Some((hex_bytes(&after_start[..end]), &after_start[end + 1..]))
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    super::super::hex::decode_hex_bytes(hex.as_bytes())
}
