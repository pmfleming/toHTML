pub(super) use super::super::hex::decode_hex_bytes;

pub(super) fn decode_pdf_string(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xfe, 0xff]) {
        return decode_utf16be(&bytes[2..]);
    }

    bytes.iter().copied().filter_map(readable_byte).collect()
}

fn decode_utf16be(bytes: &[u8]) -> String {
    let code_units = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]));
    char::decode_utf16(code_units)
        .filter_map(Result::ok)
        .collect()
}

fn readable_byte(byte: u8) -> Option<char> {
    match byte {
        b'\n' | b'\r' | b'\t' => Some(' '),
        0x20..=0x7e => Some(char::from(byte)),
        _ => None,
    }
}

pub(super) fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn is_readable_text(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }

    let meaningful = text.chars().filter(|ch| ch.is_ascii_alphanumeric()).count();
    let visible = text.chars().filter(|ch| !ch.is_whitespace()).count();
    let structural = text
        .chars()
        .any(|ch| matches!(ch, '<' | '>' | '[' | ']' | '/' | '+' | '-' | '.' | ':'));

    visible > 0 && (meaningful * 2 >= visible || (meaningful > 0 && structural) || structural)
}
