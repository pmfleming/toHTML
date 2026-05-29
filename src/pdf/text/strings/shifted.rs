// TODO: These shifted-subset repairs are compatibility fallbacks. Remove or
// narrow this layer as standard PDF font/CMap/embedded-font decoding solutions
// are implemented.
mod downshift;
mod mixed;

use super::encoding::decode_shifted_subset_text;
use super::scoring::{shifted_beats_decoded, text_score};
pub(super) use mixed::repair_mixed_shifted_subset_word;
use mixed::shifted_candidate_is_useful;

use downshift::{
    is_likely_plain_identifier, is_plain_numeric_token, repair_downshifted_subset_word,
    repair_mixed_fiscal_period_word, repair_shifted_numeric_word, split_outer_punctuation,
    starts_with_plain_acronym_before_digits,
};

pub(super) use downshift::{
    repair_downshifted_connectors, repair_downshifted_fiscal_period_sequences,
};

pub(super) fn looks_structural(text: &str) -> bool {
    text.chars()
        .any(|ch| matches!(ch, '[' | ']' | '<' | '>' | '{' | '}'))
}

pub(super) fn looks_shifted_subset_prose(text: &str) -> bool {
    if text.len() < 4 {
        return false;
    }
    text.chars().any(char::is_whitespace) || has_shifted_subset_marker(text)
}

pub(super) fn has_shifted_subset_marker(text: &str) -> bool {
    if has_shifted_subset_punctuation_marker(text) {
        return true;
    }
    let shifted_punctuation = text
        .chars()
        .filter(|ch| matches!(ch, '$' | '&' | '\'' | '(' | ')' | '+' | ',' | '<'))
        .count();
    if shifted_punctuation >= 3 && !text.chars().any(|ch| ch.is_ascii_lowercase()) {
        return true;
    }
    if text
        .chars()
        .any(|ch| matches!(ch, '[' | ']' | '<' | '>' | '{' | '}'))
    {
        return false;
    }
    if text.chars().any(|ch| ch.is_ascii_lowercase()) {
        return false;
    }
    let shifted = decode_shifted_subset_text(text.as_bytes());
    shifted != text && text_score(&shifted) > text_score(text)
}

fn has_shifted_subset_punctuation_marker(text: &str) -> bool {
    let has_marker = text
        .chars()
        .any(|ch| matches!(ch, '$' | '&' | '\'' | '%' | '*' | '+'));
    let has_upper = text.chars().any(|ch| ch.is_ascii_uppercase());
    let has_lower = text.chars().any(|ch| ch.is_ascii_lowercase());

    has_marker && has_upper && !has_lower
}

pub(super) fn repair_shifted_subset_word(word: &str) -> String {
    if let Some(marker) = repair_shifted_symbol_marker_word(word) {
        return marker;
    }
    if !word.chars().any(|ch| ch.is_ascii_alphanumeric()) {
        return word.to_string();
    }
    if let Some(period) = repair_mixed_fiscal_period_word(word) {
        return period;
    }
    if is_plain_fiscal_quarter_label(word) {
        return word.to_string();
    }
    if is_plain_numeric_token(word) {
        return word.to_string();
    }
    if is_likely_plain_identifier(word) {
        return word.to_string();
    }
    if let Some(number) = repair_shifted_numeric_word(word) {
        return number;
    }
    let (_, core, _) = split_outer_punctuation(word);
    if starts_with_plain_acronym_before_digits(core) {
        return word.to_string();
    }
    if let Some(text) = repair_downshifted_subset_word(word) {
        return text;
    }
    if contains_plain_decimal_run(word) {
        return word.to_string();
    }
    if word.len() <= 6 && word.chars().all(|ch| ch.is_ascii_uppercase()) {
        return word.to_string();
    }
    if word.chars().any(|ch| ch.is_ascii_lowercase())
        && !has_shifted_subset_punctuation_marker(word)
    {
        return word.to_string();
    }
    if word == "DW" {
        return "at".to_string();
    }
    if let Some(short) = repair_short_shifted_word(word) {
        return short.to_string();
    }
    if word.is_ascii()
        && has_shifted_subset_marker(word)
        && !word.chars().any(|ch| ch.is_ascii_lowercase())
    {
        let shifted = decode_shifted_subset_text(word.as_bytes());
        let whole_word_shift_works = if looks_structural(word) {
            shifted_beats_decoded(&shifted, word, 0)
        } else {
            shifted != word && text_score(&shifted) >= text_score(word)
        };
        if whole_word_shift_works {
            return shifted;
        }
    }
    if has_shifted_subset_marker(word) && !word.is_ascii() {
        return repair_mixed_shifted_subset_word(word);
    }
    if word.is_ascii()
        && has_shifted_subset_marker(word)
        && word.chars().any(|ch| ch.is_ascii_lowercase())
    {
        return repair_mixed_shifted_subset_word(word);
    }
    if looks_structural(word) && has_shifted_subset_marker(word) {
        return repair_mixed_shifted_subset_word(word);
    }
    if (looks_structural(word) && !has_shifted_subset_marker(word))
        || !looks_shifted_subset_prose(word)
    {
        return word.to_string();
    }

    let shifted = decode_shifted_subset_text(word.as_bytes());
    let required_gain = if has_shifted_subset_marker(word) {
        0
    } else {
        8
    };
    if shifted_beats_decoded(&shifted, word, required_gain) || shifted_candidate_is_useful(word) {
        shifted
    } else {
        word.to_string()
    }
}

fn repair_shifted_symbol_marker_word(word: &str) -> Option<String> {
    if word == "Ł" {
        return Some("•".to_string());
    }
    if let Some(stem) = word.strip_suffix("––") {
        if stem.chars().last().is_some_and(char::is_alphabetic) {
            return Some(format!("{stem}......"));
        }
    }

    let repaired = word.replace(">&", "(").replace(">'", ")");
    (repaired != word).then_some(repaired)
}

fn contains_plain_decimal_run(word: &str) -> bool {
    word.chars().any(|ch| ch.is_ascii_alphabetic())
        && word.chars().any(|ch| ch.is_ascii_digit())
        && word.chars().any(|ch| matches!(ch, '.' | ','))
}

fn is_plain_fiscal_quarter_label(word: &str) -> bool {
    let (_, core, _) = split_outer_punctuation(word);
    let mut chars = core.chars();
    matches!(chars.next(), Some('Q'))
        && matches!(chars.next(), Some('1'..='4'))
        && chars.next().is_none()
}

fn repair_short_shifted_word(word: &str) -> Option<&'static str> {
    SHORT_SHIFTED_WORDS
        .iter()
        .find_map(|(shifted, repaired)| (*shifted == word).then_some(*repaired))
}

#[rustfmt::skip]
const SHORT_SHIFTED_WORDS: &[(&str, &str)] = &[("DQ", "any"), ("DQ\\", "any"), ("EH", "be"), ("EXW", "but"), ("LQ", "in"), ("LW", "it"), ("QR", "not"), ("RI", "of"), ("RU", "or"), ("WR", "to")];
