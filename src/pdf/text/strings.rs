pub(super) use super::super::hex::decode_hex_bytes;

pub(super) fn decode_pdf_string(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xfe, 0xff]) {
        return decode_utf16be(&bytes[2..]);
    }

    bytes.iter().copied().filter_map(pdf_doc_char).collect()
}

pub(super) fn decode_pdf_text_string(bytes: &[u8]) -> String {
    let decoded = decode_pdf_string(bytes);
    if looks_structural(&decoded) {
        return decoded;
    }
    if !looks_shifted_subset_prose(&decoded) {
        return decoded;
    }
    let shifted = decode_shifted_subset_text(bytes);

    if text_score(&shifted) > text_score(&decoded) + 8 {
        shifted
    } else {
        decoded
    }
}

pub(super) fn repair_shifted_subset_words(text: &str) -> String {
    text.split_whitespace()
        .map(repair_shifted_subset_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn looks_structural(text: &str) -> bool {
    text.chars()
        .any(|ch| matches!(ch, '[' | ']' | '<' | '>' | '{' | '}'))
}

fn looks_shifted_subset_prose(text: &str) -> bool {
    if text.len() < 4 {
        return false;
    }
    text.chars().any(char::is_whitespace)
        || [
            "7KLV",
            "DQG",
            "WKH",
            "VKDOO",
            "KDYLQJ",
            "$JUHHPHQW",
            "HTXLSPHQW",
        ]
        .iter()
        .any(|marker| text.contains(marker))
}

fn repair_shifted_subset_word(word: &str) -> String {
    if looks_structural(word) || !looks_shifted_subset_prose(word) {
        return word.to_string();
    }

    let shifted = decode_shifted_subset_text(word.as_bytes());
    if text_score(&shifted) > text_score(word) + 8 {
        shifted
    } else {
        word.to_string()
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

fn pdf_doc_char(byte: u8) -> Option<char> {
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

fn decode_shifted_subset_text(bytes: &[u8]) -> String {
    bytes
        .iter()
        .copied()
        .filter_map(|byte| match byte {
            b'\n' | b'\r' | b'\t' | b' ' => Some(' '),
            0x21..=0x61 => Some(char::from(byte + 29)),
            _ => pdf_doc_char(byte),
        })
        .collect()
}

fn text_score(text: &str) -> i32 {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|ch: char| !ch.is_alphanumeric())
                .to_ascii_lowercase()
        })
        .filter(|word| !word.is_empty())
        .collect();
    let common = words.iter().filter(|word| common_word(word)).count() as i32;
    let lower = text.to_ascii_lowercase();
    let embedded_common = [
        "agreement",
        "confidential",
        "equipment",
        "information",
        "party",
        "shall",
        "the",
        "this",
    ]
    .iter()
    .filter(|word| lower.contains(**word))
    .count() as i32;
    let vowel_words = words
        .iter()
        .filter(|word| {
            word.chars()
                .any(|ch| matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u'))
        })
        .count() as i32;
    let suspicious = words
        .iter()
        .filter(|word| {
            word.len() >= 8
                && !word
                    .chars()
                    .any(|ch| matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u'))
        })
        .count() as i32;
    let weird = text
        .chars()
        .filter(|ch| matches!(ch, '}' | ']' | '^' | '~' | '\u{fffd}'))
        .count() as i32;

    common * 12 + embedded_common * 6 + vowel_words * 3 - suspicious * 8 - weird * 10
}

fn common_word(word: &str) -> bool {
    matches!(
        word,
        "a" | "an"
            | "and"
            | "are"
            | "as"
            | "be"
            | "by"
            | "for"
            | "from"
            | "in"
            | "is"
            | "it"
            | "not"
            | "of"
            | "or"
            | "shall"
            | "the"
            | "this"
            | "to"
            | "with"
            | "agreement"
            | "confidential"
            | "equipment"
            | "information"
            | "party"
    )
}

pub(super) fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn is_readable_text(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    if text
        .chars()
        .any(|ch| ch.is_control() && !ch.is_whitespace())
    {
        return false;
    }

    let meaningful = text.chars().filter(|ch| ch.is_alphanumeric()).count();
    let visible = text.chars().filter(|ch| !ch.is_whitespace()).count();
    let structural = text
        .chars()
        .any(|ch| matches!(ch, '<' | '>' | '[' | ']' | '/' | '+' | '-' | '.' | ':'));

    visible > 0 && (meaningful * 2 >= visible || structural)
}
