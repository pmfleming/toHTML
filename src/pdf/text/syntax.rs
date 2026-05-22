pub(super) fn is_text_showing_operator(word: &str) -> bool {
    matches!(word, "Tj" | "TJ" | "'" | "\"")
}

pub(super) fn is_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace()
        || matches!(byte, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'/' | b'%')
}
