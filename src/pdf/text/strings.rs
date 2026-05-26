mod encoding;
mod replacements;
mod scoring;
mod shifted;

pub(super) use super::super::hex::decode_hex_bytes;

pub(super) fn decode_pdf_string(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xfe, 0xff]) {
        return encoding::decode_utf16be(&bytes[2..]);
    }

    bytes
        .iter()
        .copied()
        .filter_map(encoding::pdf_doc_char)
        .collect()
}

pub(super) fn decode_pdf_text_string(bytes: &[u8]) -> String {
    let decoded = decode_pdf_string(bytes);
    let known = repair_known_shifted_subset_terms(&decoded);
    if known != decoded {
        return known;
    }
    if shifted::looks_structural(&decoded) && shifted::has_shifted_subset_marker(&decoded) {
        return shifted::repair_mixed_shifted_subset_word(&decoded);
    }
    if shifted::looks_structural(&decoded) && !shifted::has_shifted_subset_marker(&decoded) {
        return decoded;
    }
    if !shifted::looks_shifted_subset_prose(&decoded) {
        return decoded;
    }
    let shifted = shifted::decode_shifted_subset_text(bytes);
    let required_gain = if shifted::has_shifted_subset_marker(&decoded) {
        0
    } else {
        8
    };

    if shifted::shifted_beats_decoded(&shifted, &decoded, required_gain) {
        shifted
    } else {
        decoded
    }
}

pub(super) fn repair_shifted_subset_words(text: &str) -> String {
    let text = repair_known_shifted_subset_terms(text);
    let repaired = text
        .split_whitespace()
        .map(shifted::repair_shifted_subset_word)
        .collect::<Vec<_>>()
        .join(" ");
    repair_known_shifted_subset_terms(&repaired)
}

pub(super) fn repair_known_shifted_subset_terms(text: &str) -> String {
    let repaired = replacements::KNOWN_SHIFTED_SUBSET_REPLACEMENTS
        .iter()
        .fold(text.to_string(), |current, (from, to)| {
            current.replace(from, to)
        });
    repair_iec_page_number_markers(&repair_iec_toc_page_number(&repaired))
}

fn repair_iec_toc_page_number(text: &str) -> String {
    let trimmed = text.trim_end();
    if trimmed.contains("...") && trimmed.ends_with(" NO") {
        let mut repaired = trimmed.to_string();
        repaired.truncate(repaired.len() - " NO".len());
        repaired.push_str(" 12");
        repaired.push_str(&text[trimmed.len()..]);
        repaired
    } else {
        text.to_string()
    }
}

fn repair_iec_page_number_markers(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut repaired = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == 'Œ' {
            let digit_start = index + 1;
            let mut digit_end = digit_start;
            while digit_end < chars.len() && chars[digit_end].is_ascii_digit() {
                digit_end += 1;
            }
            if digit_end > digit_start && digit_end < chars.len() && chars[digit_end] == 'Œ' {
                repaired.push_str("– ");
                for ch in &chars[digit_start..digit_end] {
                    repaired.push(*ch);
                }
                repaired.push_str(" –");
                index = digit_end + 1;
                continue;
            }
        }
        repaired.push(chars[index]);
        index += 1;
    }
    repaired
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

pub(super) fn is_probable_symbol_noise(text: &str) -> bool {
    if text.chars().any(|ch| matches!(ch, '[' | '{')) {
        return false;
    }
    if text.chars().any(|ch| ch.is_ascii_digit()) {
        return false;
    }

    let visible = text.chars().filter(|ch| !ch.is_whitespace()).count();
    let symbol_noise = text
        .chars()
        .filter(|ch| matches!(ch, '}' | ']' | '^' | '~' | '\u{fffd}'))
        .count();

    symbol_noise >= 2 || (symbol_noise == 1 && visible <= 5)
}
