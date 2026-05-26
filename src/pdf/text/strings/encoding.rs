pub(super) fn decode_utf16be(bytes: &[u8]) -> String {
    let code_units = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]));
    char::decode_utf16(code_units)
        .filter_map(Result::ok)
        .collect()
}

pub(super) fn pdf_doc_char(byte: u8) -> Option<char> {
    match byte {
        b'\n' | b'\r' | b'\t' => Some(' '),
        0x20..=0x7e => Some(char::from(byte)),
        0x80 => Some('•'),
        0x81 => Some('†'),
        0x82 => Some('‡'),
        0x83 => Some('…'),
        0x84 => Some('—'),
        0x85 => Some('–'),
        0x86 => Some('ƒ'),
        0x87 => Some('⁄'),
        0x88 => Some('‹'),
        0x89 => Some('›'),
        0x8a => Some('−'),
        0x8b => Some('‰'),
        0x8c => Some('„'),
        0x8d => Some('“'),
        0x8e => Some('”'),
        0x8f => Some('‘'),
        0x90 => Some('’'),
        0x91 => Some('‚'),
        0x92 => Some('™'),
        0x93 => Some('ﬁ'),
        0x94 => Some('ﬂ'),
        0x95 => Some('Ł'),
        0x96 => Some('Œ'),
        0x97 => Some('Š'),
        0x98 => Some('Ÿ'),
        0x99 => Some('Ž'),
        0x9a => Some('ı'),
        0x9b => Some('ł'),
        0x9c => Some('œ'),
        0x9d => Some('š'),
        0x9e => Some('ž'),
        0xa0..=0xff => Some(char::from(byte)),
        _ => None,
    }
}
