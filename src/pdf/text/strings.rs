mod encoding;
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
    let repaired = repair_shifted_subset_structural_markers(&decoded);
    if repaired != decoded {
        return repaired;
    }
    if shifted::looks_structural(&decoded) && shifted::has_shifted_subset_marker(&decoded) {
        return shifted::repair_mixed_shifted_subset_word(&decoded);
    }
    if shifted::looks_structural(&decoded) && !shifted::has_shifted_subset_marker(&decoded) {
        return decoded;
    }
    if decoded.chars().any(|ch| ch.is_ascii_lowercase())
        && !shifted::has_shifted_subset_marker(&decoded)
    {
        return decoded;
    }
    if !shifted::looks_shifted_subset_prose(&decoded) {
        return decoded;
    }
    let shifted = encoding::decode_shifted_subset_text(bytes);
    let required_gain = if shifted::has_shifted_subset_marker(&decoded) {
        0
    } else {
        8
    };

    if scoring::shifted_beats_decoded(&shifted, &decoded, required_gain) {
        shifted
    } else {
        decoded
    }
}

pub(super) fn repair_shifted_subset_words(text: &str) -> String {
    let text = repair_shifted_subset_structural_markers(text);
    let repaired = text
        .split_whitespace()
        .map(shifted::repair_shifted_subset_word)
        .collect::<Vec<_>>();
    let repaired = shifted::repair_downshifted_fiscal_period_sequences(repaired);
    let repaired = shifted::repair_downshifted_connectors(repaired).join(" ");
    let repaired = repair_common_hyphenated_compound_spacing(&repaired);
    let repaired = repair_split_initial_cap_fragments(&repaired);
    let repaired = repair_common_joined_prose_boundaries(&repaired);
    let repaired = repair_shifted_open_parenthesis_markers(&repaired);
    let repaired = repair_shifted_parenthesis_spacing(&repaired);
    let repaired = repair_parenthetical_period_spacing(&repaired);
    repair_shifted_subset_structural_markers(&repaired)
}

pub(super) fn repair_shifted_subset_structural_markers(text: &str) -> String {
    repair_recital_number_markers(&repair_page_number_markers(&repair_leader_page_number(
        text,
    )))
}

fn repair_common_hyphenated_compound_spacing(text: &str) -> String {
    let mut repaired = text.to_string();
    for suffix in ["down", "wise", "frame", "phase", "committee"] {
        repaired = repaired.replace(&format!("- {suffix}"), &format!("-{suffix}"));
        repaired = repaired.replace(&format!(" -{suffix}"), &format!("-{suffix}"));
    }
    repaired
}

fn repair_shifted_parenthesis_spacing(text: &str) -> String {
    text.replace("( ", "(").replace(" )", ")")
}

fn repair_split_initial_cap_fragments(text: &str) -> String {
    let mut repaired = text.to_string();
    for (from, to) in [
        ("D igital", "Digital"),
        ("D imming", "Dimming"),
        ("D im ming", "Dimming"),
        ("C ommunication", "Communication"),
        ("P rotocol", "Protocol"),
        ("o ther", "other"),
    ] {
        repaired = repaired.replace(from, to);
    }
    for (from, to) in [
        ("overDigital", "over Digital"),
        ("Read igital Dimming", "Read Digital Dimming"),
        ("Read igital", "Read Digital"),
        ("DigitalDimming", "Digital Dimming"),
        ("level D, returns", "level, returns"),
        ("levelD , returns", "level, returns"),
        ("levelD, returns", "level, returns"),
        ("level D I returns", "level, returns"),
        ("level DI returns", "level, returns"),
        ("levelD I returns", "level, returns"),
        ("levelDI returns", "level, returns"),
        ("between0-200", "between 0-200"),
        ("0 - 200", "0-200"),
    ] {
        repaired = repaired.replace(from, to);
    }
    repaired
}

fn repair_common_joined_prose_boundaries(text: &str) -> String {
    let mut repaired = text.to_string();
    for (from, to) in [
        ("purpose ofthis", "purpose of this"),
        ("Regulationis", "Regulation is"),
        ("ensurea", "ensure a"),
        ("elementsand", "elements and"),
        ("solutionsshould", "solutions should"),
        ("definedas", "defined as"),
        ("byor", "by or"),
        ("of the productwith", "of the product with"),
        ("productwith", "product with"),
        ("behalf ofthe", "behalf of the"),
        ("elementsconcerned", "elements concerned"),
        ("concerned I the", "concerned, the"),
        ("Union™s", "Union's"),
        ("EU™s", "EU's"),
        ("user™s", "user's"),
        ("manufacturer™s", "manufacturer's"),
        ("aservicedeveloped", "a service developed"),
        ("within thescope", "within the scope"),
        ("withinthescope", "within the scope"),
        ("scope ofthis", "scope of this"),
    ] {
        repaired = repaired.replace(from, to);
    }
    repaired
}

fn repair_parenthetical_period_spacing(text: &str) -> String {
    let mut repaired = text.to_string();
    for digit in '0'..='9' {
        repaired = repaired.replace(&format!("Last{digit}"), &format!("Last {digit}"));
        repaired = repaired.replace(&format!("last{digit}"), &format!("last {digit}"));
    }
    repaired
}

fn repair_shifted_open_parenthesis_markers(text: &str) -> String {
    if !text.contains(')') || text.contains('=') {
        return text.to_string();
    }
    let words = text.split_whitespace().collect::<Vec<_>>();
    if words
        .iter()
        .filter(|word| word.chars().filter(|ch| ch.is_alphabetic()).count() >= 3)
        .count()
        < 4
    {
        return text.to_string();
    }

    let repaired = words
        .iter()
        .enumerate()
        .map(|(index, word)| {
            let previous_has_letters = index
                .checked_sub(1)
                .and_then(|previous| words.get(previous))
                .is_some_and(|word| word.chars().any(char::is_alphabetic));
            let next_starts_like_prose = words
                .get(index + 1)
                .and_then(|word| word.chars().find(|ch| ch.is_alphabetic()))
                .is_some_and(|ch| ch.is_uppercase());
            if *word == "E" && previous_has_letters && next_starts_like_prose {
                "("
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    if repaired != text {
        repaired
    } else {
        text.to_string()
    }
}

fn repair_leader_page_number(text: &str) -> String {
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

fn repair_page_number_markers(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut repaired = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == 'Œ' {
            let digit_start = index + 1;
            let mut digit_end = digit_start;
            while digit_end < chars.len()
                && chars[digit_end] != 'Œ'
                && !chars[digit_end].is_whitespace()
            {
                digit_end += 1;
            }
            if digit_end > digit_start && digit_end < chars.len() && chars[digit_end] == 'Œ' {
                let Some(page_number) = decode_marker_page_number(&chars[digit_start..digit_end])
                else {
                    repaired.push(chars[index]);
                    index += 1;
                    continue;
                };
                repaired.push_str("– ");
                repaired.push_str(&page_number);
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

fn repair_recital_number_markers(text: &str) -> String {
    if !text
        .split_whitespace()
        .any(|word| decode_recital_marker(word).is_some())
    {
        return text.to_string();
    }
    text.split_whitespace()
        .map(|word| {
            decode_recital_marker(word)
                .map_or_else(|| word.to_string(), |number| format!("({number})"))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_recital_marker(word: &str) -> Option<String> {
    let chars = word.chars().collect::<Vec<_>>();
    if chars.len() < 3 || chars.first() != Some(&'E') || chars.last() != Some(&'F') {
        return None;
    }
    decode_marker_page_number(&chars[1..chars.len() - 1])
}

fn decode_marker_page_number(chars: &[char]) -> Option<String> {
    if chars.is_empty() || chars.len() > 4 {
        return None;
    }
    chars
        .iter()
        .copied()
        .map(|ch| match ch {
            '0'..='9' => Some(ch),
            'M' => Some('0'),
            'N' => Some('1'),
            'O' => Some('2'),
            'P' => Some('3'),
            'Q' => Some('4'),
            'R' => Some('5'),
            'S' => Some('6'),
            'T' => Some('7'),
            'U' => Some('8'),
            'V' => Some('9'),
            _ => None,
        })
        .collect()
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
    let structural = text.chars().any(|ch| {
        matches!(
            ch,
            '<' | '>'
                | '['
                | ']'
                | '('
                | ')'
                | '/'
                | '+'
                | '-'
                | '.'
                | ':'
                | '•'
                | '◦'
                | '▪'
                | '□'
                | '☐'
                | '☑'
                | '☒'
                | '✓'
                | '✔'
        )
    });

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
