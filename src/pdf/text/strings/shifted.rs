use super::encoding::decode_shifted_subset_text;
use super::scoring::{shifted_beats_decoded, text_score};

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

pub(super) fn repair_downshifted_connectors(mut words: Vec<String>) -> Vec<String> {
    for index in 0..words.len() {
        if words[index] != "C" {
            continue;
        }
        let previous = index
            .checked_sub(1)
            .and_then(|previous| words.get(previous))
            .is_some_and(|word| looks_like_short_downshifted_label(word));
        let next = words
            .get(index + 1)
            .is_some_and(|word| looks_like_short_downshifted_label(word));
        if previous && next {
            words[index] = "&".to_string();
        }
    }
    words
}

pub(super) fn repair_downshifted_fiscal_period_sequences(words: Vec<String>) -> Vec<String> {
    let mut repaired = Vec::with_capacity(words.len());
    let mut index = 0;
    while index < words.len() {
        if let Some(period) = words
            .get(index..index + 2)
            .and_then(repair_downshifted_fiscal_period_sequence)
        {
            repaired.push(period);
            index += 2;
            continue;
        }
        if let Some(period) = words
            .get(index..index + 1)
            .and_then(repair_downshifted_fiscal_period_sequence)
        {
            repaired.push(period);
            index += 1;
            continue;
        }

        repaired.push(words[index].clone());
        index += 1;
    }
    repaired
}

fn repair_downshifted_fiscal_period_sequence(words: &[String]) -> Option<String> {
    if words.is_empty()
        || !words
            .iter()
            .any(|word| word.chars().any(can_downshift_subset_char))
    {
        return None;
    }

    let mut combined = String::new();
    let mut prefix = "";
    let mut suffix = "";
    for (index, word) in words.iter().enumerate() {
        let (word_prefix, core, word_suffix) = split_outer_punctuation(word);
        if index == 0 {
            prefix = word_prefix;
        } else if !word_prefix.is_empty() {
            return None;
        }
        if index + 1 == words.len() {
            suffix = word_suffix;
        } else if !word_suffix.is_empty() {
            return None;
        }
        combined.push_str(core);
    }

    let repaired = downshift_subset_text(&combined);
    if looks_like_fiscal_period_expression(&repaired)
        || looks_like_fiscal_half_label(&repaired)
        || looks_like_prefixed_fiscal_half_label(&repaired)
    {
        Some(format!("{prefix}{repaired}{suffix}"))
    } else {
        None
    }
}

fn looks_like_fiscal_period_expression(text: &str) -> bool {
    let chars = text.chars().collect::<Vec<_>>();
    chars.len() == 9
        && chars[0] == 'H'
        && matches!(chars[1], '1' | '2')
        && chars[2] == '('
        && chars[3] == 'Q'
        && matches!(chars[4], '1'..='4')
        && chars[5] == '+'
        && chars[6] == 'Q'
        && matches!(chars[7], '1'..='4')
        && chars[8] == ')'
}

fn looks_like_fiscal_half_label(text: &str) -> bool {
    let chars = text.chars().collect::<Vec<_>>();
    chars.len() == 6
        && chars[0] == '2'
        && chars[1] == '0'
        && chars[2].is_ascii_digit()
        && chars[3].is_ascii_digit()
        && chars[4] == 'H'
        && matches!(chars[5], '1' | '2')
}

fn looks_like_prefixed_fiscal_half_label(text: &str) -> bool {
    text.char_indices().skip(1).any(|(index, _)| {
        let (prefix, label) = text.split_at(index);
        !prefix.is_empty()
            && prefix.chars().all(is_formula_prefix_char)
            && looks_like_fiscal_half_label(label)
    })
}

fn is_formula_prefix_char(ch: char) -> bool {
    matches!(ch, '=' | '(' | '[' | '<' | '>' | '/' | '≤' | '≥')
}

fn is_plain_numeric_token(word: &str) -> bool {
    let trimmed = word.trim_matches(|ch: char| matches!(ch, ',' | ';' | ':' | '(' | ')'));
    if trimmed.is_empty() {
        return false;
    }

    let without_trailing_period = trimmed
        .strip_suffix('.')
        .filter(|value| !value.contains('.'))
        .unwrap_or(trimmed);
    let numeric = without_trailing_period
        .strip_prefix('-')
        .unwrap_or(without_trailing_period);
    if numeric.is_empty() {
        return false;
    }

    let decimal_separator = if numeric.contains(',') { ',' } else { '.' };
    let mut parts = numeric.split(decimal_separator);
    let Some(whole) = parts.next() else {
        return false;
    };
    if whole.is_empty() || !whole.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }

    match (parts.next(), parts.next()) {
        (None, None) => true,
        (Some(fraction), None) => {
            !fraction.is_empty() && fraction.chars().all(|ch| ch.is_ascii_digit())
        }
        _ => false,
    }
}

fn is_likely_plain_identifier(word: &str) -> bool {
    let has_digit = word.chars().any(|ch| ch.is_ascii_digit());
    let allowed = word.chars().all(|ch| {
        ch.is_ascii_uppercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_' | '.' | '/' | '$')
    });
    word.len() >= 4 && has_digit && allowed
}

fn repair_shifted_numeric_word(word: &str) -> Option<String> {
    let (prefix, core, suffix) = split_outer_punctuation(word);
    if core.len() < 3 || core.len() > 5 {
        return None;
    }
    if core.chars().filter(|ch| *ch == 'I').count() != 1 {
        return None;
    }
    let mut repaired = String::new();
    for ch in core.chars() {
        match ch {
            'I' => {
                if repaired.is_empty() {
                    repaired.push('0');
                }
                repaired.push(',');
            }
            'M' => repaired.push('0'),
            'N' => repaired.push('1'),
            'O' => repaired.push('2'),
            'P' => repaired.push('3'),
            'Q' => repaired.push('4'),
            'R' => repaired.push('5'),
            'S' => repaired.push('6'),
            'T' => repaired.push('7'),
            'U' => repaired.push('8'),
            'V' => repaired.push('9'),
            _ => return None,
        }
    }
    let (whole, fraction) = repaired.split_once(',')?;
    if whole.is_empty() || fraction.is_empty() {
        return None;
    }
    Some(format!("{prefix}{repaired}{suffix}"))
}

fn repair_downshifted_subset_word(word: &str) -> Option<String> {
    let (prefix, core, suffix) = split_outer_punctuation(word);
    if core.is_empty() || !core.chars().any(can_downshift_subset_char) {
        return None;
    }
    if is_plain_short_uppercase_acronym(core) && !is_shifted_quarter_fragment(core) {
        return None;
    }
    if starts_with_plain_acronym_before_digits(core) {
        return None;
    }

    let repaired = downshift_subset_text(core);
    if repaired == core {
        return None;
    }
    if !looks_like_downshifted_number_or_label(&repaired) {
        return None;
    }

    Some(format!("{prefix}{repaired}{suffix}"))
}

fn is_plain_short_uppercase_acronym(text: &str) -> bool {
    (2..=3).contains(&text.len()) && text.chars().all(|ch| ch.is_ascii_uppercase())
}

fn is_shifted_quarter_fragment(text: &str) -> bool {
    matches!(text, "QP" | "QQ")
}

fn starts_with_plain_acronym_before_digits(text: &str) -> bool {
    for split_at in 2..=3 {
        let Some((prefix, suffix)) = text
            .char_indices()
            .nth(split_at)
            .map(|(index, _)| text.split_at(index))
        else {
            continue;
        };
        if prefix.chars().all(|ch| ch.is_ascii_uppercase())
            && suffix.chars().next().is_some_and(|ch| ch.is_ascii_digit())
        {
            return true;
        }
    }
    false
}

fn repair_mixed_fiscal_period_word(word: &str) -> Option<String> {
    let (prefix, core, suffix) = split_outer_punctuation(word);
    let repaired = repair_mixed_fiscal_period_core(core)?;
    Some(format!("{prefix}{repaired}{suffix}"))
}

fn repair_mixed_fiscal_period_core(core: &str) -> Option<String> {
    let mut chars = core.char_indices();
    let (_, first) = chars.next()?;
    let (second_index, second) = chars.next()?;
    if first != 'e' || !second.is_ascii_digit() {
        return None;
    }

    let rest = &core[second_index + second.len_utf8()..];
    if !rest.starts_with('(') || !(rest.contains("Q3") || rest.contains("Q4")) {
        return None;
    }

    let mut repaired = String::with_capacity(core.len());
    repaired.push('H');
    repaired.push(second);
    repaired.push_str(rest);
    Some(repaired)
}

fn downshift_subset_text(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if can_downshift_subset_char(ch) {
                char::from(ch as u8 - 29)
            } else {
                ch
            }
        })
        .collect()
}

fn can_downshift_subset_char(ch: char) -> bool {
    ch.is_ascii() && matches!(ch as u8, b'B'..=b'Z' | b'a'..=b'z')
}

fn looks_like_downshifted_number_or_label(text: &str) -> bool {
    looks_like_downshifted_number(text) || looks_like_short_downshifted_label(text)
}

fn looks_like_downshifted_number(text: &str) -> bool {
    let numeric = text.strip_prefix('-').unwrap_or(text);
    if numeric.len() < 2 {
        return false;
    }
    let mut parts = numeric.split('.');
    let Some(whole) = parts.next() else {
        return false;
    };
    if whole.is_empty() || !valid_grouped_digits(whole) {
        return false;
    }
    match (parts.next(), parts.next()) {
        (None, None) => true,
        (Some(fraction), None) => {
            !fraction.is_empty() && fraction.chars().all(|ch| ch.is_ascii_digit())
        }
        _ => false,
    }
}

fn looks_like_short_downshifted_label(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(chars.next(), Some('A'..='Z'))
        && chars.all(|ch| ch.is_ascii_digit())
        && (2..=3).contains(&text.len())
}

fn valid_grouped_digits(text: &str) -> bool {
    if text.chars().all(|ch| ch.is_ascii_digit()) {
        return true;
    }
    let groups = text.split(',').collect::<Vec<_>>();
    let Some((first, rest)) = groups.split_first() else {
        return false;
    };
    !first.is_empty()
        && first.chars().all(|ch| ch.is_ascii_digit())
        && rest
            .iter()
            .all(|group| group.len() == 3 && group.chars().all(|ch| ch.is_ascii_digit()))
}

fn split_outer_punctuation(word: &str) -> (&str, &str, &str) {
    let start = word
        .char_indices()
        .find(|(_, ch)| ch.is_ascii_alphanumeric())
        .map(|(index, _)| index)
        .unwrap_or(word.len());
    let end = word
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_ascii_alphanumeric())
        .map(|(index, ch)| index + ch.len_utf8())
        .unwrap_or(start);
    (&word[..start], &word[start..end], &word[end..])
}

fn repair_short_shifted_word(word: &str) -> Option<&'static str> {
    SHORT_SHIFTED_WORDS
        .iter()
        .find_map(|(shifted, repaired)| (*shifted == word).then_some(*repaired))
}

#[rustfmt::skip]
const SHORT_SHIFTED_WORDS: &[(&str, &str)] = &[("DQ", "any"), ("DQ\\", "any"), ("EH", "be"), ("EXW", "but"), ("LQ", "in"), ("LW", "it"), ("QR", "not"), ("RI", "of"), ("RU", "or"), ("WR", "to")];

pub(super) fn repair_mixed_shifted_subset_word(word: &str) -> String {
    let mut repaired = String::new();
    let mut chunk = String::new();
    let mut shifted_chunk = None;

    for ch in word.chars() {
        let is_shifted = is_shifted_subset_byte(ch);
        match shifted_chunk {
            Some(current) if current == is_shifted => chunk.push(ch),
            Some(_) => {
                repaired.push_str(&repair_shifted_subset_chunk(&chunk));
                chunk.clear();
                chunk.push(ch);
                shifted_chunk = Some(is_shifted);
            }
            None => {
                chunk.push(ch);
                shifted_chunk = Some(is_shifted);
            }
        }
    }

    if !chunk.is_empty() {
        repaired.push_str(&repair_shifted_subset_chunk(&chunk));
    }

    repaired
}

fn repair_shifted_subset_chunk(chunk: &str) -> String {
    if !chunk.is_ascii() {
        return chunk.to_string();
    }
    if !chunk.chars().any(is_shifted_subset_byte) {
        return chunk.to_string();
    }
    if chunk.chars().all(|ch| ch.is_ascii_alphabetic())
        && !has_shifted_subset_marker(chunk)
        && !shifted_candidate_is_useful(chunk)
    {
        return chunk.to_string();
    }

    let shifted = decode_shifted_subset_text(chunk.as_bytes());
    if shifted != chunk
        && (text_score(&shifted) >= text_score(chunk) || shifted_candidate_is_useful(chunk))
    {
        shifted
    } else {
        chunk.to_string()
    }
}

fn is_shifted_subset_byte(ch: char) -> bool {
    ch.is_ascii() && ('!'..='`').contains(&ch)
}

fn shifted_candidate_is_useful(chunk: &str) -> bool {
    let shifted = decode_shifted_subset_text(chunk.as_bytes()).to_ascii_lowercase();
    USEFUL_SHIFTED_TERMS
        .split_whitespace()
        .any(|word| shifted.contains(word))
}

const USEFUL_SHIFTED_TERMS: &str = "agreement agreements changed either made other accountants accounts agents applicable authorized available breach contained covenant certain confidential consideration conclusion construed contract copies disclosing disclosure documents delivered destroyed developed drawings employees exchange efforts furnished forth information including independently instructions limitation limited mutual officers party parties process property provided produced product promptly prevent purpose receiving relating remain representatives requisitions results specifications shall sole such survive the therefore this transaction under warranties will with written";
