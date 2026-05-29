mod plain;

pub(in crate::pdf::text::strings::shifted) use plain::{
    is_likely_plain_identifier, is_plain_numeric_token, starts_with_plain_acronym_before_digits,
};

pub(in crate::pdf::text::strings::shifted) fn repair_shifted_numeric_word(
    word: &str,
) -> Option<String> {
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

pub(in crate::pdf::text::strings::shifted) fn repair_downshifted_subset_word(
    word: &str,
) -> Option<String> {
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

pub(in crate::pdf::text::strings::shifted::downshift) fn downshift_subset_text(
    text: &str,
) -> String {
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

pub(in crate::pdf::text::strings::shifted::downshift) fn can_downshift_subset_char(
    ch: char,
) -> bool {
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

pub(super) fn looks_like_short_downshifted_label(text: &str) -> bool {
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

pub(in crate::pdf::text::strings::shifted) fn split_outer_punctuation(
    word: &str,
) -> (&str, &str, &str) {
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
