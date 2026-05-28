pub fn unicode_string(bytes: &[u8]) -> String {
    if bytes.len() >= 2 {
        return repair_private_use_symbols(&decode_utf16be(bytes));
    }
    repair_private_use_symbols(
        &bytes
            .iter()
            .copied()
            .filter_map(readable_byte)
            .collect::<String>(),
    )
}

pub fn unicode_scalar(value: u32) -> String {
    char::from_u32(value)
        .map(String::from)
        .map(|text| repair_private_use_symbols(&text))
        .unwrap_or_default()
}

pub fn readable_byte(byte: u8) -> Option<char> {
    match byte {
        b'\n' | b'\r' | b'\t' => Some(' '),
        0x20..=0x7e => Some(char::from(byte)),
        _ => None,
    }
}

fn decode_utf16be(bytes: &[u8]) -> String {
    let code_units = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]));
    char::decode_utf16(code_units)
        .filter_map(Result::ok)
        .collect()
}

fn repair_private_use_symbols(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '\u{f070}' => '□',
            _ => ch,
        })
        .collect()
}
